use std::str::SplitWhitespace;

use tokio::io;

use crate::{
    connection::{Connection, Mail, State},
    parser::{extract_email, responses::OK},
};

pub fn mail(
    connection: &mut Connection,
    mut command: SplitWhitespace<'_>,
    domain: String,
) -> Result<&'static [u8], io::Error> {
    log::info!("Command received: MAIL");
    match command.next() {
        Some(email) => {
            let extracted_email = extract_email(email);

            if let Some(email) = extracted_email {
                connection.state = State::MailFrom(Mail {
                    from: email.to_owned(),
                    domain,
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
    Ok(OK)
}
