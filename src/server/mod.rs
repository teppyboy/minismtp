mod create;
mod start;

use std::{marker::PhantomData, path::PathBuf, time::Duration};

use async_std::channel::{Receiver, RecvError, SendError, Sender};
use thiserror::Error;
use tokio::{io, task::JoinError};

use crate::connection::Mail;

#[derive(Error, Debug)]
/**
## Server error
The `ServerError` enum represents an error that can occur while running the SMTP server.
*/
pub enum ServerError {
    #[error("Network error: {0}")]
    /**
     * Occurs when there is a general network error
     */
    Network(#[from] io::Error),
    #[error("Task error: {0}")]
    /**
     * Occurs when there is an error with am async task
     */
    Task(#[from] JoinError),
    #[error("Send error: {0}")]
    /**
     * Occurs when there is an error transmitting a signal or an email
     */
    Send(#[from] SendError<()>),
    #[error("Recv error: {0}")]
    /**
     * Occurs when there is an error receiving a signal or an email
     */
    Recv(#[from] RecvError),
    #[error("Could not bind to {host}:{port} because {source}")]
    /**
     * Occurs when the server cannot bind to a host and port
     */
    Bind {
        host: &'static str,
        port: u16,
        #[source]
        source: io::Error,
    },
    #[error("Could not confirm shutdown")]
    /**
     * Occurs when the server cannot confirm a shutdown via signalling
     */
    Shutdown,
    #[error("Server is already running")]
    /**
     * Occurs when the server is already running and a start is attempted
     */
    Running,
}

#[derive(Debug, Clone)]
/**
## Configuration for the SMTP server

   The configuration for the SMTP server includes the following fields:
   - `host`: The host on which the server will listen for incoming connections.
   - `port`: The port on which the server will listen for incoming connections.
   - `domain`: The domain of the server.
   - `timeout`: The duration after which the server will timeout.
   - `buffer_size`: The size of the buffer used for reading incoming data (bytes).
   - `certs_path`: The path to the certificates used for encryption.
   - `key_path`: The path to the keys used for encryption.
   - `mail_tx`: The sender for the mail channel.
   - `affirm_tx`: The sender for the affirmation channel.
   - `shutdown_rx`: The receiver for the shutdown channel.
*/
pub struct Config {
    pub host: &'static str,
    pub port: u16,
    pub domain: &'static str,
    pub timeout: Option<Duration>,
    pub buffer_size: Option<usize>,
    pub certs_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub mail_tx: Sender<Mail>,
    pub affirm_tx: Sender<()>,
    pub shutdown_rx: Receiver<()>,
}

/**
    Listening state for the server
*/
pub struct Listening;

/**
    Closed state for the server
*/
pub struct Closed;

/**
## The SMTP server

   The SMTP server is a struct that represents the server itself. It includes the following fields:
   - `config`: The configuration for the server.
   - `mail_rx`: The receiver for the mail channel.
   - `affirm_rx`: The receiver for the affirmation channel.
   - `shutdown_tx`: The sender for the shutdown channel.
   - `state`: The state of the server.

   The server can be in one of two states: `Closed` or `Listening`.

   The `Closed` state indicates that the server is not running, while the `Listening` state indicates that the server is running and listening for incoming connections.

   After creating an instance of the server, you can start it by calling the `start` method, which will spawn a task to start the server and return a `Result` indicating whether the server was started successfully.
   The returned value will be an instance of the SmtpServer struct, with a listener that can be used to receive incoming emails.

   Example:
   ```rust
    use minismtp::server::SmtpServer;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::main]
    async fn main() {
        let server = SmtpServer::new(
            "localhost",
            2525,
            "localhost",
            Some(Duration::from_secs(10)),
            None,
            None,
            None,
        );

        let listening_server = server.start().await.unwrap();

        // Actually send an email to the server and do something with this
        // returned value.
        let _ = timeout(Duration::from_secs(5), listening_server.mail_rx.recv()).await;

        listening_server.stop().await.unwrap();
    }

   ```
*/
pub struct SmtpServer<State = Closed> {
    config: Config,
    pub mail_rx: Receiver<Mail>,
    affirm_rx: Receiver<()>,
    shutdown_tx: Sender<()>,
    state: PhantomData<State>,
}
