mod responses;

use responses::{EHLO_TLS_AVAILABLE, EHLO_TLS_UNAVAILABLE, OK, READY_FOR_TLS, TLS_NOT_AVAILABLE};
// Custom error with thiserror
use thiserror::Error;
use tokio::io;

use crate::connection::{Connection, Mail, State, TlsConfig};

fn extract_email(email: &str) -> Option<&str> {
    let chars = email.chars().enumerate();
    let mut at = false; // Tracks presence of '@'
    let mut start = None;
    let mut end = None;

    for (i, c) in chars {
        match c {
            '<' => start = Some(i + 1), // Start after '<'
            '>' => {
                end = Some(i); // End before '>'
                break; // No need to look further beyond '>'
            }
            '@' => {
                if start.is_some() && end.is_none() {
                    at = true;
                }
            }
            _ => {}
        }
    }

    if let (Some(st), Some(en)) = (start, end) {
        if at && st < en {
            // Additional checks to ensure indices and presence of '@'
            return Some(&email[st..en]);
        }
    }
    None
}

pub fn parse_and_execute(
    connection: &mut Connection,
    command: String,
) -> Result<Vec<u8>, io::Error> {
    log::info!("SMTP Processor: Processing command {:?}", command);

    let mut command = command.split_whitespace();

    match (command.next(), connection.state.clone()) {
        (Some("ehlo"), State::Initial) => {
            log::info!("SMTP Processor: EHLO command received");
            log::info!("SMTP Processor: Sending 250 response");
            connection.state = State::Ehlo;

            Ok(match connection.tls_config {
                TlsConfig::Encrypted { .. } => EHLO_TLS_AVAILABLE.as_bytes().to_vec(),
                _ => EHLO_TLS_UNAVAILABLE.as_bytes().to_vec(),
            })
        }
        (Some("starttls"), State::Ehlo) => {
            log::info!("SMTP Processor: STARTTLS command received");
            Ok(match connection.tls_config {
                TlsConfig::Encrypted { .. } => {
                    connection.state = State::StartTls;
                    READY_FOR_TLS.as_bytes().to_vec()
                }
                _ => TLS_NOT_AVAILABLE.as_bytes().to_vec(),
            })
        }
        (Some("mail"), State::Ehlo) => {
            log::info!("SMTP Processor: MAIL command received");
            match command.next() {
                Some(email) => {
                    let extracted_email = extract_email(email);

                    if let Some(email) = extracted_email {
                        connection.state = State::MailFrom(Mail {
                            from: email.to_owned(),
                            ..Default::default()
                        });
                        log::info!("Sender: {:?}", email);
                    } else {
                        connection.state = State::Invalid;
                        log::error!("Invalid Sender");
                    }
                }
                None => {
                    connection.state = State::Invalid;
                    log::error!("Invalid Sender");
                }
            }
            Ok(OK.as_bytes().to_vec())
        }
        (Some("rcpt"), State::MailFrom(mail)) => {
            log::info!("SMTP Processor: RCPT command received");
            match command.next() {
                Some(email) => {
                    let extracted_email = extract_email(email);
                    if let Some(email) = extracted_email {
                        let mut current_recipients = mail.to.clone();
                        current_recipients.push(email.to_owned());
                        log::info!("Recipients: {:?}", current_recipients);

                        connection.state = State::MailFrom(Mail {
                            to: current_recipients,
                            ..mail.clone()
                        });
                    } else {
                        connection.state = State::Invalid;
                        log::error!("Invalid recipient");
                    }
                }
                None => {
                    connection.state = State::Invalid;
                    log::error!("Invalid recipient");
                }
            }
            Ok(OK.as_bytes().to_vec())
        }
        _ => {
            log::error!("SMTP Processor: Invalid command {:?}", command);
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid command",
            ))
        }
    }
}
