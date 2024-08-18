use std::{marker::PhantomData, path::PathBuf, rc::Rc, sync::Arc, time::Duration};

use async_std::channel::unbounded;
use tokio::{
    io::{self},
    net::TcpListener,
    task::{self, JoinError},
    time::timeout,
};

use crate::connection::{Connection, Stream};

use super::{start::start_server, Closed, Config, Listening, ServerError, SmtpServer};


impl SmtpServer {
    pub fn new(
        host: &'static str,
        port: u16,
        domain: &'static str,
        timeout: Option<Duration>,
        buffer_size: Option<usize>,
        certs_path: Option<PathBuf>,
        key_path: Option<PathBuf>,
    ) -> SmtpServer<Closed> {
        if certs_path.is_none() || key_path.is_none() {
            log::info!("No certificates or keys provided, STARTTLS will not be available.")
        }

        let (mail_tx, mail_rx) = unbounded();
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
                shutdown_rx
            },
        }
    }

    pub async fn start(self) -> Result<SmtpServer<Listening>, ServerError> {
        task::spawn(start_server(self.config.clone()));
        log::info!("Requested server start");
        self.affirm_rx.recv().await?;
        log::info!("Confirmed server start");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start() {
        //let server = SmtpServer::new("locassslhost", 4000, Duration::from_secs(5));

        //server.start().await.unwrap();
    }
}
