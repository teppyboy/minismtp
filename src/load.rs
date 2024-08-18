use std::{fs::File, io::BufReader, path::PathBuf};

use rustls_pemfile::{certs, private_key};
use thiserror::Error;
use tokio::io;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Io error: {0}")]
    Network(#[from] io::Error),
    #[error("No private key found")]
    PrivateKey,
}

pub fn load_certs(path: PathBuf) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

pub fn load_key(path: PathBuf) -> Result<PrivateKeyDer<'static>, LoadError> {
    private_key(&mut BufReader::new(File::open(path)?))?
        .ok_or(LoadError::PrivateKey)
}
