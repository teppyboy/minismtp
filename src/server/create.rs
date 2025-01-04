use std::{marker::PhantomData, path::PathBuf, time::Duration};

use async_std::channel::unbounded;
use tokio::task;

use crate::connection::Mail;

use super::{start::start_server, Closed, Config, Listening, ServerError, SmtpServer};

impl SmtpServer {
    /**
    ## Create a new SMTP server instance

       The `new` function creates a new SMTP server instance with the following parameters:
       - `host`: The host on which the server will listen for incoming connections.
       - `port`: The port on which the server will listen for incoming connections.
       - `domain`: The domain of the server.
       - `timeout`: The duration after which the server will timeout.
       - `buffer_size`: The size of the buffer used for reading incoming data (bytes).
       - `certs_path`: The path to the certificates used for encryption.
       - `key_path`: The path to the keys used for encryption.

       The function returns a new SMTP server instance.

       # Example

       ```rust
       use minismtp::server::SmtpServer;
       use std::time::Duration;

       #[tokio::main]
       async fn main() {
           let server = SmtpServer::new(
               "localhost",
               2525,
               "localhost",
               Some(Duration::from_secs(10)),
               Some(1024),
               None,
               None,
           );
       }
       ```
    */
    pub fn new(
        host: String,
        port: u16,
        domain: String,
        timeout: Option<Duration>,
        buffer_size: Option<usize>,
        certs_path: Option<PathBuf>,
        key_path: Option<PathBuf>,
    ) -> SmtpServer<Closed> {
        if certs_path.is_none() || key_path.is_none() {
            log::info!("No certificates or keys provided, STARTTLS will not be available.")
        }

        let (mail_tx, mail_rx) = unbounded::<Mail>();
        let (affirm_tx, affirm_rx) = unbounded();
        let (shutdown_tx, shutdown_rx) = unbounded();

        SmtpServer {
            mail_rx,
            affirm_rx,
            shutdown_tx,
            state: PhantomData::<Closed>,
            config: Config {
                host,
                port,
                domain,
                timeout,
                buffer_size,
                certs_path,
                key_path,
                mail_tx,
                affirm_tx,
                shutdown_rx,
            },
        }
    }

    /**
    Starts the server. Returns an error if server could not start
    */
    pub async fn start(self) -> Result<SmtpServer<Listening>, ServerError> {
        task::spawn(start_server(self.config.clone()));
        log::info!("Requesting server start...");
        self.affirm_rx.recv().await?;
        log::info!("Server started.");
        Ok(SmtpServer {
            state: PhantomData::<Listening>,
            config: self.config.clone(),
            mail_rx: self.mail_rx.clone(),
            affirm_rx: self.affirm_rx.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
        })
    }
}

impl SmtpServer<Listening> {
    /**
    Stops the server. Returns an error if server could not stop.
    */
    pub async fn stop(self) -> Result<SmtpServer<Closed>, ServerError> {
        self.shutdown_tx.send(()).await?;
        self.affirm_rx.recv().await?;

        Ok(SmtpServer {
            state: PhantomData::<Closed>,
            config: self.config.clone(),
            mail_rx: self.mail_rx.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
            affirm_rx: self.affirm_rx.clone(),
        })
    }
}
