mod process;
mod create;
mod rw;
use std::{path::PathBuf, sync::Arc};

use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use thiserror::Error;
use tokio::io;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Could not respond to client")]
    Response,
    #[error("Could not read from client")]
    Read,
}


#[derive(Debug,Clone,PartialEq)]
pub enum State {
    Initial,
    Ehlo,
    StartTls,
    MailFrom(Mail),
    RcptTo(Mail),
    Invalid

}


#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mail {
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
pub enum TlsConfig {
    Plain,
    Encrypted{
        cert_path:PathBuf,
        key_path:PathBuf,
    },
}


#[derive(Debug)]
pub struct Connection {
    pub buffer_size: Option<usize>,
    pub stream: Stream,
    pub state: State,
    pub tls_config: TlsConfig,
    pub domain:&'static str,
}

