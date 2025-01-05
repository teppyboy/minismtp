use std::slice::Split;
use tokio::io;

use crate::{
    connection::{Connection, State, TlsConfig},
    parser::responses::{EHLO_TLS_AVAILABLE, EHLO_TLS_UNAVAILABLE},
};

pub fn ehlo(
    connection: &mut Connection,
    mut command: Split<'_, u8, impl FnMut(&u8) -> bool>
) -> Result<&'static [u8], io::Error> {
    log::info!("Command received: EHLO");
    log::info!("Sending 250 response");
    // Read the domain from the command
    if let Some(domain) = command.next() {
        let domain_str = std::str::from_utf8(domain).unwrap();
        log::info!("Domain: {}", domain_str);
        connection.state = State::Ehlo(domain_str.to_string());
    } else {
        connection.state = State::Ehlo("".to_string());
    }
    // Return based on the TLS configuration
    Ok(match connection.tls_config {
        TlsConfig::Encrypted { .. } => &EHLO_TLS_AVAILABLE,
        _ => &EHLO_TLS_UNAVAILABLE,
    })
}
