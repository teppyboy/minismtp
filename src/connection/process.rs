use super::{Connection, Mail, ProcessingError};
use crate::{
    connection::{State, Stream, TlsConfig},
    parser::{parse_and_execute, responses::QUIT},
};
use tokio::time::timeout;
use tokio_tls_upgrade::upgrade_tcp_stream;

impl Connection {
    pub async fn process_buffer(&mut self, buf: &mut [u8]) -> Result<bool, ProcessingError> {
        match self.read(buf).await {
            Ok(n) => {
                if n == 0 {
                    log::info!("Connection closed by client");
                    // Change state here later.
                    return Err(ProcessingError::ConnectionClosed);
                }

                let command = String::from_utf8_lossy(&buf[..n]).to_lowercase();
                log::info!("Received command: {:?}", command);
                let result = parse_and_execute(self, command)?;

                if !result.is_empty() {
                    if let Ok(()) = self.write(result).await {
                        if result == QUIT {
                            log::info!("Closing connection");
                            return Ok(false);
                        }
                        return Ok(true);
                    } else {
                        return Err(ProcessingError::SendResponse);
                    }
                }
                log::info!("Sending response: {:?}", &String::from_utf8_lossy(result));
                Ok(true)
            }
            Err(e) => {
                log::error!("Error reading from socket: {}", e);
                Err(ProcessingError::SocketRead)
            }
        }
    }

    pub async fn process(mut self) -> Result<Mail, ProcessingError> {
        self.greet().await?;

        log::info!("Greeting sent");

        // This expression multiplies 1024 by 1024, resulting in 1,048,576 bytes, which is equivalent to 1 megabyte (MB).
        let mut buf = vec![0; self.buffer_size.unwrap_or(1024 * 1024)];

        loop {
            log::info!("Waiting for data...");
            match timeout(self.timeout, self.process_buffer(&mut buf)).await {
                Ok(Ok(keep_open)) => {
                    if !keep_open {
                        break;
                    }
                }
                Ok(Err(e)) => {
                    log::error!("Error processing buffer: {}", e);

                    match self.state {
                        State::Data(mail) => return Ok(mail),
                        _ => return Err(e),
                    };
                }
                Err(_) => {
                    log::error!("Connection timed out. Closing connection...");
                    break;
                }
            }

            if self.state == State::StartTls {
                log::info!("Upgrading connection to use TLS");
                match self.stream {
                    Stream::Plain(stream) => match self.tls_config {
                        TlsConfig::Plain => {
                            log::error!("TLS upgrade requested but no certificate provided");
                            return Err(ProcessingError::NoCertificate);
                        }
                        TlsConfig::Encrypted {
                            ref cert_path,
                            ref key_path,
                        } => {
                            self.stream = Stream::Encrypted(
                                upgrade_tcp_stream(stream, cert_path.clone(), key_path.clone())
                                    .await?,
                            );
                            self.state = State::Initial;
                            log::info!("Connection upgraded to TLS");
                        }
                    },
                    _ => {
                        log::error!("Cannot upgrade an already encrypted connection to TLS");
                        return Err(ProcessingError::AlreadyEncrypted);
                    }
                }
            }
        }
        match self.state {
            State::Data(mail) => Ok(mail),
            _ => Err(ProcessingError::NoMail),
        }
    }

    /// Sends the initial SMTP greeting
    async fn greet(&mut self) -> Result<(), ProcessingError> {
        self.write(format!("220 {}\r\n", self.domain).as_bytes())
            .await?;
        Ok(())
    }
}
