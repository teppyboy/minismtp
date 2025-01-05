use std::slice::Split;
use tokio::io;

use crate::{
    connection::{Connection, State},
    parser::responses::EHLO_TLS_UNAVAILABLE,
};

pub fn helo(
    connection: &mut Connection,
    mut command: Split<'_, u8, impl FnMut(&u8) -> bool>
) -> Result<&'static [u8], io::Error> {
    log::info!("Command received: HELO");
    log::info!("Sending 250 response");
    // Read the domain from the command
    if let Some(domain) = command.next() {
        let domain_str = std::str::from_utf8(domain).unwrap();
        log::info!("Domain: {}", domain_str);
        connection.state = State::Ehlo(domain_str.to_string());
    } else {
        connection.state = State::Ehlo("".to_string());
    }
    // We never support TLS on HELO
    Ok(&EHLO_TLS_UNAVAILABLE)
}
