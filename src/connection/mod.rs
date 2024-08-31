mod create;
mod process;
mod rw;
use std::{path::PathBuf, time::Duration};

use thiserror::Error;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

use crate::security::spf::SpfPolicy;

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
pub enum State {
    Initial,
    Ehlo(String),
    StartTls,
    MailFrom(Mail),
    Data(Mail),
    Invalid,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mail {
    pub domain: String,
    pub from: String,
    pub to: Vec<String>,
    pub data: String,
    pub spf: (bool, SpfPolicy),
}

#[derive(Debug)]
pub enum Stream {
    Plain(TcpStream),
    Encrypted(TlsStream<TcpStream>),
}

#[derive(Debug)]
pub enum TlsConfig {
    Plain,
    Encrypted {
        cert_path: PathBuf,
        key_path: PathBuf,
    },
}

#[derive(Debug)]
pub struct Connection {
    pub buffer_size: Option<usize>,
    pub stream: Stream,
    pub state: State,
    pub tls_config: TlsConfig,
    pub domain: &'static str,
    pub timeout: Duration,
}
