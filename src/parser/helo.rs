use std::str::SplitWhitespace;

use tokio::io;

use crate::{
    connection::{Connection, State},
    parser::responses::EHLO_TLS_UNAVAILABLE,
};

pub fn helo(
    connection: &mut Connection,
    mut command: SplitWhitespace<'_>,
) -> Result<&'static [u8], io::Error> {
    log::info!("Command received: HELO");
    log::info!("Sending 250 response");
    // Read the domain from the command
    if let Some(domain) = command.next() {
        log::info!("Domain: {}", domain);
        connection.state = State::Ehlo(domain.to_string());
    } else {
        connection.state = State::Ehlo("".to_string());
    }
    // We never support TLS on HELO
    Ok(EHLO_TLS_UNAVAILABLE)
}
