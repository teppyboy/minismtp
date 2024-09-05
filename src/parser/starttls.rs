use tokio::io;

use crate::{
    connection::{Connection, State, TlsConfig},
    parser::responses::{READY_FOR_TLS, TLS_NOT_AVAILABLE},
};

pub fn starttls(connection: &mut Connection) -> Result<&'static [u8], io::Error> {
    log::info!("Command received: STARTTLS");
    // Check if the tls configuration allows for encryption
    Ok(match connection.tls_config {
        TlsConfig::Encrypted { .. } => {
            connection.state = State::StartTls;
            READY_FOR_TLS
        }
        _ => TLS_NOT_AVAILABLE,
    })
}
