mod data;
mod ehlo;
mod helo;
mod mail;
mod rcpt;
pub mod responses;
mod starttls;

use data::{data, prepare_for_data};
use ehlo::ehlo;
use helo::helo;
use mail::mail;
use rcpt::rcpt;
use responses::QUIT;
use starttls::starttls;
use tokio::io;

use crate::connection::{Connection, State};

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
    raw_command: &[u8],
) -> Result<&'static [u8], io::Error> {
    log::info!("SMTP Processor: Processing command...");

    // Split the received data by whitespace
    let mut commands = raw_command.split(|c| *c == b' ' || *c == b'\r' || *c == b'\n');
    // let mut command: std::str::SplitWhitespace<'_> = raw_command.split_whitespace();

    // The first phrase in the command is the command itself
    // We match the command to a handler based on the current state of the connection
    let command = match commands.next() {
        Some(c) => c,
        None => {
            log::error!("Invalid command {:?}", raw_command);
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid command",
            ));
        }
    };
    
    let command_string = std::str::from_utf8(command).unwrap().to_lowercase();
    let command_str = command_string.as_str();
    log::info!("Received command: {:?}", command_str);
    match (command_str, connection.state.clone()) {
        ("ehlo", State::Initial) => ehlo(connection, commands),
        ("helo", State::Initial) => helo(connection, commands),
        ("starttls", State::Ehlo(_domain)) => starttls(connection),
        ("mail", State::Ehlo(domain)) => mail(connection, commands, domain),
        ("rcpt", State::MailFrom(mail)) => rcpt(connection, commands, mail),
        ("data", State::MailFrom(mail)) => prepare_for_data(connection, mail),
        ("quit", _) => {
            log::info!("Command received: QUIT");
            return Ok(QUIT);
        }
        (_, State::Data(mail)) => data(connection, mail, raw_command),
        _ => {
            log::error!("Invalid command {:?}", command);
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid command",
            ));
        }
    }
}
