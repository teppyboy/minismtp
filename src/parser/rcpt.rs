use std::slice::Split;

use tokio::io;

use crate::{
    connection::{Connection, Mail, State},
    parser::{extract_email, responses::OK},
};

pub fn rcpt(
    connection: &mut Connection,
    mut command: Split<'_, u8, impl FnMut(&u8) -> bool>,
    mail: Mail,
) -> Result<&'static [u8], io::Error> {
    log::info!("Command received: RCPT");
    match command.next() {
        Some(email) => {
            // Extract the email from the command
            let email_str = std::str::from_utf8(email).unwrap();
            let extracted_email = extract_email(email_str);
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
