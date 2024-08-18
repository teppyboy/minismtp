use std::{path::PathBuf, rc::Rc, sync::Arc};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::{Connection, State, Stream, TlsConfig};

impl Connection {
    pub async fn new(
        domain: &'static str,
        stream: Stream,
        cert_path: Option<PathBuf>,
        key_path: Option<PathBuf>,
        buffer_size: Option<usize>,
    ) -> Self {
        let state = State::Initial;

        let tls_config = match (cert_path, key_path) {
            (Some(cert_path), Some(key_path)) => TlsConfig::Encrypted {
                cert_path: cert_path,
                key_path: key_path,
            },
            _ => TlsConfig::Plain,
        };

        Connection {
            domain,
            stream,
            state,
            tls_config,
            buffer_size,
        }
    }
}
