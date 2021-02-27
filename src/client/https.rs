use std::io::{Error as IoError, BufReader};
use std::path::PathBuf;
use hyper::{Body, Uri, Response};
use hyper::client::{Client as HttpClient, HttpConnector};
use hyper_rustls::HttpsConnector;
use rustls::ClientConfig;
use crate::util::io::open_file;

#[derive(Debug)]
pub enum NewClientError {
    IoError(IoError),
    InvalidCertificateAuthority,
}

pub type NewClientResult = Result<Client, NewClientError>;

pub struct Client {
    client: HttpClient<HttpsConnector<HttpConnector>>,
}

impl Client {
    pub fn new(path_to_ca: Option<&PathBuf>) -> NewClientResult {
        let mut ca_reader = match path_to_ca {
            Some(path) => {
                let file = open_file(path).map_err(NewClientError::IoError)?;
                Some(BufReader::new(file))
            },
            None => None,
        };

        let https_connector = match ca_reader {
            Some(ref mut reader) => {
                let mut http = HttpConnector::new();
                http.enforce_http(false);

                let mut tls = ClientConfig::new();
                tls.root_store.add_pem_file(reader).map_err(|_| NewClientError::InvalidCertificateAuthority)?;

                HttpsConnector::from((http, tls))
            },
            None => HttpsConnector::with_native_roots(),
        };

        let client: HttpClient<_, hyper::Body> = HttpClient::builder().build(https_connector);

        Ok(
            Self {
                client
            }
        )
    }

    pub async fn get(&self, uri: Uri) -> Result<Response<Body>, hyper::Error> {
        self.client.get(uri).await
    }
}