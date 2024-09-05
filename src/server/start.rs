use std::time::Duration;

use futures::{select, FutureExt};
use tokio::net::TcpListener;

use crate::connection::{Connection, Stream};

use super::{Config, ServerError};

/**
   Waits for a connection to be established and then processes it.
*/
async fn wait_for_connection(listener: &TcpListener, config: Config) {
    match listener.accept().await {
        Ok((socket, addr)) => {
            log::info!("New connection: {}", addr);
            tokio::spawn(async move {
                // Create a new connection instance
                let connection = Connection::new(
                    config.domain,
                    Stream::Plain(socket),
                    config.certs_path.clone(),
                    config.key_path.clone(),
                    config.buffer_size,
                    config.timeout.unwrap_or(Duration::from_secs(10)),
                )
                .await;

                // Process the connection
                let process = connection.process().await;

                match process {
                    Ok(value) => {
                        // The final result should be an email that we can forward to the channel
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

/**
Starts the TCP server that listens for incoming connections.
*/
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

    // While listening for incoming connections, we also listen for shutdown signals.
    loop {
        select! {
            _ = config.shutdown_rx.recv().fuse() => {
                log::info!("Shutting down server");
                if let Err(error)=config.affirm_tx.send(()).await {
                    log::error!("Error sending shutdown confirmation: {}", error);
                }
                return Ok(());
            }
            _ = wait_for_connection(&listener,config.clone()).fuse() => {
                log::info!("Connection handled");
            }

        }
    }
}
