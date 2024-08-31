use std::time::Duration;

use futures::{select, FutureExt};
use tokio::{net::TcpListener, task::spawn_blocking};

use crate::{
    connection::{Connection, Stream},
    security::spf::{check_spf, SpfPolicy},
};

use super::{Config, ServerError};

async fn wait_for_connection(listener: &TcpListener, config: Config, timeout: Duration) {
    match listener.accept().await {
        Ok((socket, addr)) => {
            log::info!("New connection: {}", addr);
            tokio::spawn(async move {
                let connection = Connection::new(
                    config.domain,
                    Stream::Plain(socket),
                    config.certs_path.clone(),
                    config.key_path.clone(),
                    config.buffer_size,
                    timeout,
                )
                .await;

                let process = connection.process().await;

                match process {
                    Ok(mut value) => {
                        let domain = value.domain.clone();
                        value.spf = spawn_blocking(move || check_spf(addr.ip(), domain))
                            .await
                            .unwrap_or((false, SpfPolicy::Fail));
                        if let Ok(()) = config.mail_tx.send(value).await {
                            log::info!("Mail forwarded to channel");
                        } else {
                            log::error!("Error sending mail to to channel");
                        }
                    }
                    Err(e) => {
                        log::error!("Processing error: {}", e);
                    }
                }
            });
        }
        Err(e) => {
            log::error!("Task: Error accepting connection: {}", e)
        }
    }
}

pub async fn start_server(config: Config) -> Result<(), ServerError> {
    log::info!("Starting server on {}:{}", config.host, config.port);
    let listener = TcpListener::bind(&format!("{}:{}", config.host, config.port))
        .await
        .map_err(|e| ServerError::Bind {
            host: config.host,
            port: config.port,
            source: e,
        })?;
    config.affirm_tx.send(()).await?;

    let timeout = config.timeout.unwrap_or(Duration::from_secs(10));

    loop {
        select! {
            _ = config.shutdown_rx.recv().fuse() => {
                log::info!("Shutting down server");
                if let Err(error)=config.affirm_tx.send(()).await {
                    log::error!("Error sending shutdown confirmation: {}", error);
                }
                return Ok(());
            }
            _ = wait_for_connection(&listener,config.clone(),timeout).fuse() => {
                log::info!("Connection handled");
            }

        }
    }
}
