mod create;
mod process;
mod rw;
use std::{path::PathBuf, time::Duration};

use thiserror::Error;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Could not read mail")]
    NoMail,
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Could not send response")]
    SendResponse,
    #[error("No certificate provided")]
    NoCertificate,
    #[error("Already encrypted")]
    AlreadyEncrypted,
    #[error("Socket read")]
    SocketRead,
}

#[derive(Debug, Clone, PartialEq)]
/**
   ## State enum
   The `State` enum represents the state of an SMTP connection.
   It includes the following variants:
   - `Initial`: The initial state of the connection.
   - `Ehlo`: The state after the EHLO command has been received.
   - `StartTls`: The state after the STARTTLS command has been received.
   - `MailFrom`: The state after the MAIL FROM command has been received.
   - `Data`: The state after the DATA command has been received.
   - `Invalid`: An invalid state.

*/
pub enum State {
    Initial,
    Ehlo(String),
    StartTls,
    MailFrom(Mail),
    Data(Mail),
    Invalid,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/**
## Mail struct
   The `Mail` struct represents an email message.
   It includes the following fields:
   - `domain`: The domain of the email.
   - `from`: The sender of the email.
   - `to`: The recipients of the email.
   - `data`: The raw content of the email, including headers and body.
*/
pub struct Mail {
    pub domain: String,
    pub from: String,
    pub to: Vec<String>,
    pub data: String,
}

#[derive(Debug)]
pub enum Stream {
    Plain(TcpStream),
    Encrypted(TlsStream<TcpStream>),
}

#[derive(Debug)]
/**
## TLS Configuration
   The `TlsConfig` enum represents the configuration for TLS encryption.
   It includes the following variants:
   - `Plain`: Represents a plain connection without encryption.
   - `Encrypted`: Represents an encrypted connection with the provided certificate and key paths.

   This enum is used to configure the connection to use TLS encryption.
*/
pub enum TlsConfig {
    Plain,
    Encrypted {
        cert_path: PathBuf,
        key_path: PathBuf,
    },
}

#[derive(Debug)]
/**
   ## Connection struct
   The `Connection` struct represents an SMTP connection.
   It includes the following fields:
   - `buffer_size`: The size of the buffer used for reading incoming data (bytes).
   - `stream`: The stream used for the connection.
   - `state`: The state of the connection.
   - `tls_config`: The TLS configuration for the connection.
   - `domain`: The domain of the connection.
   - `timeout`: The duration after which the connection will timeout.
*/
pub struct Connection {
    pub buffer_size: Option<usize>,
    pub stream: Stream,
    pub state: State,
    pub tls_config: TlsConfig,
    pub domain: &'static str,
    pub timeout: Duration,
}
