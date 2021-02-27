use std::sync::Arc;
use log::{info, debug, warn};
use hyper::{Body, Request, Response, Error};
use hyper::body::{Bytes, to_bytes};
use hyper::http::HeaderValue;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::fs::create_dir_all;
use regex::Regex;
use crate::config::{Mirrors, Mirror};
use crate::client::https::Client;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

pub async fn proxy(request: Request<Body>, client: Arc<Client>, mirrors: Arc<Mirrors>) -> Result<Response<Body>, Error> {
    let uri = request.uri().clone();

    let response = client.get(uri.clone()).await?;

    let (mut parts, mut body) = response.into_parts();
    let bytes = to_bytes(&mut body).await.unwrap();

    let host = uri.host().unwrap_or("127.0.0.1");
    let mirror = mirrors.get(host);

    match mirror_content(mirror, uri.path(), &bytes).await {
        Ok(()) => (),
        Err(why) => {
            warn!("Could not save mirrored path {} for host {}: {:?}", uri.path(), host, why);
        }
    };

    let (header_name, header_value) = make_proxy_header();
    parts.headers.insert(header_name, header_value);

    Ok(
        Response::from_parts(parts, Body::from(bytes))
    )
}

async fn mirror_content(maybe_mirror: Option<&Mirror>, path: &str, content: &Bytes) -> Result<(), std::io::Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^/]+\.[^.]+$").expect("Infallible");
    }

    let mirror = match maybe_mirror {
        Some(mirror) => mirror,
        None => return Ok(()),
    };

    let file_part = match RE.find(path) {
        Some(part) => part.as_str(),
        None => return Ok(()),
    };

    let is_allowed = {
        let mut allowed = false;

        for extension in &mirror.extensions {
            if file_part.contains(extension.as_str()) {
                allowed = true;
                break;
            }
        }

        allowed
    };

    if !is_allowed {
        return Ok(())
    }

    let (directory_path, absolute_path) = {
        let mut clear_path = RE.replace(path, "").to_string();
        clear_path.remove(0);

        let mut directory_path = mirror.location.clone();
        directory_path.push(&clear_path);

        let mut absolute_path = directory_path.clone();
        absolute_path.push(&file_part);

        (directory_path, absolute_path)
    };

    if !directory_path.exists() {
        debug!("Expanding: {:?}", &directory_path);
        create_dir_all(&directory_path).await?;
    }

    if absolute_path.exists() {
        debug!("Path {:?} already exists, not saving.", absolute_path);
        return Ok(());
    }

    info!("Mirror {:?} found file {:?}. Saving to {:?}.", mirror.name, file_part, directory_path);

    File::create(absolute_path)
        .await?
        .write_all(content)
        .await?;

    Ok(())
}

fn make_proxy_header() -> (&'static str, HeaderValue) {
    let raw_line = format!("yes; https-proxy-rs v.{}", VERSION.unwrap_or("unknown"));

    ("X-Proxy-Server", HeaderValue::from_str(raw_line.as_str()).expect("Infallible"))
}