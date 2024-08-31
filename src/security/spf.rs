use hickory_resolver::Resolver;
use ipnetwork::IpNetwork;
use std::{collections::HashMap, env, net::IpAddr};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum SpfPolicy {
    Pass,
    #[default]
    Fail,
    SoftFail,
    Neutral,
    None,
}

fn determine_spf_policy(record: &str) -> SpfPolicy {
    if let Some(policy_part) = record.split_whitespace().last() {
        match policy_part {
            "+all" | "all" => SpfPolicy::Pass,
            "-all" => SpfPolicy::Fail,
            "~all" => SpfPolicy::SoftFail,
            "?all" => SpfPolicy::Neutral,
            _ => SpfPolicy::None,
        }
    } else {
        SpfPolicy::None
    }
}

fn get_spf(
    ips: &mut HashMap<String, SpfPolicy>,
    resolver: Resolver,
    domain: String,
    depth: usize,
) -> &mut HashMap<String, SpfPolicy> {
    if depth > 10 {
        log::error!(
            "Max recursion depth reached when querying SPF record for domain: {}",
            domain
        );
        return ips;
    }
    match resolver.txt_lookup(&domain) {
        Ok(txt) => {
            for record in txt.iter() {
                let record = record.to_string();
                if record.contains("v=spf1") {
                    let spf_policy = determine_spf_policy(&record);
                    let parts = record.split_whitespace().collect::<Vec<&str>>();
                    for part in parts.iter() {
                        match *part {
                            // Handling includes:
                            directive if directive.starts_with("include:") => {
                                let include_domain = &directive["include:".len()..];
                                return get_spf(
                                    ips,
                                    resolver,
                                    include_domain.to_owned(),
                                    depth + 1,
                                );
                            }
                            // Direct IP4 and IP6 includes:
                            directive if directive.starts_with("ip4:") => {
                                let ip = &directive["ip4:".len()..];
                                ips.insert(ip.to_string(), spf_policy.clone());
                            }
                            directive if directive.starts_with("ip6:") => {
                                let ip = &directive["ip6:".len()..];
                                ips.insert(ip.to_string(), spf_policy.clone());
                            }
                            _ => continue, // Ignore unrecognized parts or mechanism like `~all`
                        }
                    }
                }
            }
            ips
        }
        Err(_) => {
            log::info!("No TXT record found for domain: {}", domain);
            ips
        }
    }
}

pub fn check_spf(ip: IpAddr, domain: String) -> (bool, SpfPolicy) {
    let ip = {
        if let Ok(override_ip) = env::var("SPF_IP") {
            override_ip.parse::<IpAddr>().unwrap_or_else(|_| {
                log::error!("Error parsing IP from SPF_IP environment variable");
                ip
            })
        } else {
            ip
        }
    };
    let domain = env::var("SPF_DOMAIN").unwrap_or(domain);
    let mut ips: HashMap<String, SpfPolicy> = HashMap::new();

    match Resolver::from_system_conf() {
        Ok(resolver) => {
            let raw_ips = get_spf(&mut ips, resolver, domain, 0);

            let mut cidrs: HashMap<IpNetwork, SpfPolicy> = HashMap::new();

            for ip in raw_ips.iter() {
                match ip.0.parse() {
                    Ok(netmask) => {
                        cidrs.insert(netmask, ip.1.clone());
                    }
                    Err(error) => {
                        log::error!(
                            "Error parsing IP from SPF record: {} because {}",
                            ip.0,
                            error
                        );
                    }
                }
            }

            for cidr in cidrs.iter() {
                if cidr.0.contains(ip) {
                    log::info!("IP {} is in SPF record for domain. SPF Check Passed.", ip);
                    return (true, cidr.1.clone());
                }
            }

            log::info!(
                "IP {} is not in SPF record for domain. SPF Check Failed.",
                ip
            );
            (false, SpfPolicy::Fail)
        }
        Err(_) => {
            log::error!("Error creating resolver");
            (false, SpfPolicy::Fail)
        }
    }
}
