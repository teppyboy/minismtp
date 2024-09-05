use std::{path::PathBuf, time::Duration};

use super::{Connection, State, Stream, TlsConfig};

impl Connection {
    /**
       ## New method
       The `new` method creates a new `Connection` instance.
       It takes the following arguments:
       - `domain`: The domain of the connection.
       - `stream`: The stream used for the connection.
       - `cert_path`: An optional path to the certificate file.
       - `key_path`: An optional path to the key file.
       - `buffer_size`: An optional buffer size for reading incoming data.
       - `timeout`: The duration after which the connection will timeout.

       It returns a new `Connection` instance.
    */
    pub async fn new(
        domain: &'static str,
        stream: Stream,
        cert_path: Option<PathBuf>,
        key_path: Option<PathBuf>,
        buffer_size: Option<usize>,
        timeout: Duration,
    ) -> Self {
        let state = State::Initial;

        let tls_config = match (cert_path, key_path) {
            (Some(cert_path), Some(key_path)) => TlsConfig::Encrypted {
                cert_path,
                key_path,
            },
            _ => TlsConfig::Plain,
        };

        Connection {
            domain,
            stream,
            state,
            tls_config,
            buffer_size,
            timeout,
        }
    }
}
