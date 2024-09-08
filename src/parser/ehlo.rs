use std::str::SplitWhitespace;
use tokio::io;

use crate::{
    connection::{Connection, State, TlsConfig},
    parser::responses::{EHLO_TLS_AVAILABLE, EHLO_TLS_UNAVAILABLE},
};

pub fn ehlo(
    connection: &mut Connection,
    mut command: SplitWhitespace<'_>,
) -> Result<&'static [u8], io::Error> {
    log::info!("Command received: EHLO");
    log::info!("Sending 250 response");
    // Read the domain from the command
    if let Some(domain) = command.next() {
        log::info!("Domain: {}", domain);
        connection.state = State::Ehlo(domain.to_string());
    } else {
        connection.state = State::Ehlo("".to_string());
    }
    // Return based on the TLS configuration
    Ok(match connection.tls_config {
        TlsConfig::Encrypted { .. } => &EHLO_TLS_AVAILABLE,
        _ => &EHLO_TLS_UNAVAILABLE,
    })
}
