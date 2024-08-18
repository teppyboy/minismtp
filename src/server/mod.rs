mod create;
mod start;

use std::{marker::PhantomData, path::PathBuf, rc::Rc, sync::Arc, time::Duration};

use async_std::channel::{Receiver, RecvError, SendError, Sender};
use thiserror::Error;
use tokio::{io, task::JoinError};

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Network error: {0}")]
    Network(#[from] io::Error),
    #[error("Task error: {0}")]
    Task(#[from] JoinError),
    #[error("Send error: {0}")]
    Send(#[from] SendError<()>),
    #[error("Recv error: {0}")]
    Recv(#[from] RecvError),
    #[error("Could not bind to {host}:{port} because {source}")]
    Bind {
        host: &'static str,
        port: u16,
        #[source]
        source: io::Error,
    },
    #[error("Could not confirm shutdown")]
    Shutdown,
    #[error("Server is already running")]
    Running,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: &'static str,
    pub port: u16,
    pub domain: &'static str,
    pub timeout: Option<Duration>,
    pub buffer_size: Option<usize>,
    pub certs_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub mail_tx: Sender<()>,
    pub affirm_tx: Sender<()>,
    pub shutdown_rx: Receiver<()>,
}

pub struct Listening;
pub struct Closed;

pub struct SmtpServer<State = Closed> {
    pub config: Config,
    mail_rx: Receiver<()>,
    affirm_rx: Receiver<()>,
    shutdown_tx: Sender<()>,
    state: PhantomData<State>,
}
