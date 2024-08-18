use tokio_tls_upgrade::upgrade_tcp_stream;
use crate::{connection::{State, Stream, TlsConfig}, parser::parse_and_execute};
use super::{Connection, ProcessingError};


impl Connection {

    pub async fn process(mut self) -> Result<(),ProcessingError> {
        self.greet().await?;

        log::info!("Transmission: Greeting sent");

        // This expression multiplies 1024 by 1024, resulting in 1,048,576 bytes, which is equivalent to 1 megabyte (MB).
        let mut buf = vec![0; self.buffer_size.unwrap_or(1024 * 1024)];

        loop {
            match self.read(&mut buf).await {
                Ok(n) => {
                    if n == 0 {
                        log::info!("Transmission: Connection closed by client");
                        // Change state here later.
                        break;
                    }
                    let command = String::from_utf8_lossy(&buf[..n]).to_lowercase();
                    log::info!("Transmission: Received command: {}", command);
                    let result = parse_and_execute(&mut self, command)?;
                    
                    if !result.is_empty() {
                        self.write(&result).await?;
                    }

                    if self.state == State::StartTls {
                        log::info!("Connection: Upgrading connection to TLS");
                        match self.stream {
                            Stream::Plain(stream) => {

                                match self.tls_config {
                                    TlsConfig::Plain => {
                                        log::error!("Connection: TLS upgrade requested but no certificate provided");
                                        break;
                                    }
                                    TlsConfig::Encrypted { ref cert_path, ref key_path } => {
                                        self.stream = Stream::Encrypted(
                                            upgrade_tcp_stream(
                                                stream,
                                                cert_path.clone(),
                                                key_path.clone(),
                                            )
                                            .await?,
                                        );
                                        self.state = State::Initial;
                                        log::info!("Connection: Connection upgraded to TLS");
                                    }
                                }
                            }
                            _ => {
                                log::error!("Connection: Cannot upgrade an already encrypted connection to TLS");
                                break;
                            }
                        }
                    }

                    log::info!(
                        "Transmission: Sending response: {:?}",
                        String::from_utf8_lossy(&result)
                    );
                }
                Err(e) => {
                    log::error!("Connection: Error reading from socket: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }


    /// Sends the initial SMTP greeting
    async fn greet(&mut self) -> Result<(), ProcessingError> {
        self.write(format!("220 {}\n", self.domain).as_bytes())
            .await?;
        Ok(())
    }
}