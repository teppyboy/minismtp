use std::time::Duration;

use tokio::{net::TcpListener, time::timeout};

use crate::connection::{Connection, Stream};

use super::{Config, ServerError};

pub async fn start_server(config: Config) -> Result<(), ServerError> {
    log::info!("Task: Starting server on {}:{}", config.host, config.port);
    let listener = TcpListener::bind(&format!("{}:{}", config.host, config.port))
        .await
        .map_err(|e| ServerError::Bind {
            host: config.host,
            port: config.port,
            source: e,
        })?;
    log::info!("Task: Server started on {}:{}", config.host, config.port);
    log::info!("Task: Sending affirmation");
    config.affirm_tx.send(()).await?;
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                log::info!("Task: New connection: {}", addr);
                let connection = Connection::new(
                    config.domain,
                    Stream::Plain(socket),
                    config.certs_path.clone(),
                    config.key_path.clone(),
                    config.buffer_size,
                )
                .await;
                let process = timeout(
                    config.timeout.unwrap_or(Duration::from_secs(600)),
                    connection.process(),
                )
                .await;

                if let Err(e) = process {
                    log::error!("Task: Connection: Error processing connection: {}", e);
                }
            }
            Err(e) => {
                log::error!("Task: Error accepting connection: {}", e)
            }
        }
    }
}
