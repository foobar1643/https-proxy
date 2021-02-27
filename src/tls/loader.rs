use std::io::{Result, BufReader};
use std::path::PathBuf;
use rustls::{Certificate, PrivateKey};
use rustls::internal::pemfile;
use crate::util::io::{open_file, make_io_error};

pub fn load_certs(path: &PathBuf) -> Result<Vec<Certificate>> {
    let cert_file = open_file(path)?;
    let mut reader = BufReader::new(cert_file);

    pemfile::certs(&mut reader).map_err(|_| make_io_error("Failed to load certificates to memory".into()))
}

pub fn load_private_key(path: &PathBuf) -> Result<PrivateKey> {
    let key_file = open_file(path)?;
    let mut reader = BufReader::new(key_file);

    let keys = pemfile::rsa_private_keys(&mut reader)
        .map_err(|_| make_io_error("Failed to load private key to memory".into()))?;

    keys.get(0)
        .cloned()
        .ok_or_else(|| make_io_error("Could not locate private key".into()))
}