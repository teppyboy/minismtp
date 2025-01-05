use tokio::io;

use crate::{
    connection::{Connection, Mail, State},
    parser::responses::{OK, SEND_DATA},
};

pub fn prepare_for_data(
    connection: &mut Connection,
    mail: Mail,
) -> Result<&'static [u8], io::Error> {
    log::info!("Command received: DATA");
    log::info!("Awaiting data...");
    connection.state = State::Data(mail);

    Ok(SEND_DATA)
}

pub fn data(
    connection: &mut Connection,
    mut mail: Mail,
    raw_command: &[u8],
) -> Result<&'static [u8], io::Error> {
    log::info!("Some data received");
    // Append the data to the mail
    mail.data.extend_from_slice(&raw_command);
    connection.state = State::Data(mail);
    if raw_command.ends_with("\r\n.\r\n".as_bytes()) {
        log::info!("Data received successfully");
        Ok(OK)
    } else {
        Ok(&[])
    }
}
