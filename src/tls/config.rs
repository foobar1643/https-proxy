use std::sync::Arc;
use std::path::PathBuf;
use std::io::Error as IoError;
use rustls::{ServerConfig, NoClientAuth, TLSError};
use crate::tls::loader::{load_certs, load_private_key};

#[derive(Debug)]
pub enum ServerConfigError {
    IoError(IoError),
    TlsError(TLSError),
}

pub type ArcServerConfig = Arc<ServerConfig>;
pub type TlsConfigResult = Result<ArcServerConfig, ServerConfigError>;

pub fn make_tls_config(certs_location: &PathBuf, private_key_location: &PathBuf) -> TlsConfigResult {
    let certificates = load_certs(certs_location)
        .map_err(ServerConfigError::IoError)?;

    let private_key = load_private_key(private_key_location)
        .map_err(ServerConfigError::IoError)?;

    let mut tls_config = ServerConfig::new(NoClientAuth::new());

    tls_config.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
    tls_config.set_single_cert(certificates, private_key)
        .map_err(ServerConfigError::TlsError)?;

    Ok(
        Arc::new(tls_config),
    )
}