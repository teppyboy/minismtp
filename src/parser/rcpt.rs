use std::str::SplitWhitespace;

use tokio::io;

use crate::{
    connection::{Connection, Mail, State},
    parser::{extract_email, responses::OK},
};

pub fn rcpt(
    connection: &mut Connection,
    mut command: SplitWhitespace<'_>,
    mail: Mail,
) -> Result<&'static [u8], io::Error> {
    log::info!("Command received: RCPT");
    match command.next() {
        Some(email) => {
            // Extract the email from the command
            let extracted_email = extract_email(email);
            if let Some(email) = extracted_email {
                // Add the recipient to the list of recipients
                let mut current_recipients = mail.to.clone();
                current_recipients.push(email.to_owned());
                log::info!("Recipients: {:?}", current_recipients);
                // Update the connection state
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
    Ok(OK)
}
