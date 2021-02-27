use std::io::Error as IoError;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use serde::Deserialize;
use crate::config::loader::{read_config, deserialize_config};

mod loader;

pub type Mirrors = HashMap<String, Mirror>;
type RawMirrors = HashMap<String, RawMirror>;

#[derive(Debug)]
pub enum LoadingError {
    IoError(IoError),
    DeserializationError(String),
}

pub fn load_config(path: &PathBuf) -> Result<Config, LoadingError> {
    let raw_cfg = read_config(path)?;
    deserialize_config::<Config>(raw_cfg.as_str())
}

#[derive(Debug, Deserialize)]
pub struct Config {
    server: Server,
    client: Client,
    #[serde(rename = "mirror")]
    raw_mirrors: RawMirrors,
}

impl Config {
    pub fn into_parts(self) -> (Server, Client, Mirrors) {
        let mut mirrors = HashMap::with_capacity(32);

        for (name, raw_mirror) in self.raw_mirrors {
            mirrors.insert(
                raw_mirror.uri,
                Mirror {
                    name,
                    extensions: raw_mirror.extensions,
                    location: raw_mirror.location,
                }
            );
        }

        (self.server, self.client, mirrors)
    }
}

#[derive(Debug, Deserialize)]
pub struct Server {
    #[serde(default = "default_addr")]
    pub addr: SocketAddr,
    pub certs_location: PathBuf,
    pub private_key_location: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Client {
    pub ca_location: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Mirror {
    pub name: String,
    pub extensions: Vec<String>,
    pub location: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct RawMirror {
    pub uri: String,
    pub extensions: Vec<String>,
    pub location: PathBuf,
}

fn default_addr() -> SocketAddr {
    "127.0.0.1:443".parse().expect("Infallible")
}