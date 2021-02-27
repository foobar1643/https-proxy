#[macro_use]
extern crate lazy_static;

use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use log::{error, debug, info, SetLoggerError};
use hyper::service::{make_service_fn, service_fn};
use fern::colors::{ColoredLevelConfig, Color};
use hyper::Server;
use tokio_rustls::TlsAcceptor;
use tokio::net::TcpListener;
use crate::config::{load_config, Mirrors};
use crate::tls::config::make_tls_config;
use crate::client::https::Client;
use crate::svc::proxy::proxy;
use crate::tls::HyperTlsAcceptor;

mod config;
mod client;
mod tls;
mod svc;
mod util;

#[tokio::main]
async fn main() {
    let (client, listener, tls_acceptor, mirrors) = startup()
        .await
        .unwrap_or_else(|why| {
            error!("Exiting. Startup has failed: {}", why);
            std::process::exit(1);
        });

    let client = Arc::new(client);
    let mirrors = Arc::new(mirrors);

    let make_svc = make_service_fn(move |_| {
        let client = client.clone();
        let mirrors = mirrors.clone();

        async move {
            Ok::<_, Infallible>(
                service_fn(
                    move |request| proxy(request, client.clone(), mirrors.clone())
                )
            )
        }
    });

    let server = Server::builder(HyperTlsAcceptor::new(listener, tls_acceptor))
        .serve(make_svc);

    if let Err(why) = server.with_graceful_shutdown(shutdown_signal()).await {
        error!("Server is shutting down: {:?}", why);
    }
}

async fn startup() -> Result<(Client, TcpListener, TlsAcceptor, Mirrors), String> {
    create_logger().unwrap_or_else(|why| {
        println!("Could not create system logger: {:?}", why);
        std::process::exit(1);
    });

    info!("https-proxy is starting up.");

    let config_location: PathBuf = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".into())
        .parse()
        .expect("Infallible");

    debug!("Path to application config is: {:?}, attempting to load.", &config_location);

    let (server_config, client_config, mirrors) = load_config(&config_location)
        .map_err(|why| format!("Could not load config: {:?}", why))?
        .into_parts();

    debug!("Application config {:?} has been loaded, attempting to create server-side TLS config.", &config_location);

    let tls_config = make_tls_config(&server_config.certs_location, &server_config.private_key_location)
        .map_err(|why| format!("Could not create TLS config: {:?}", why))?;

    let tls_acceptor = TlsAcceptor::from(tls_config);

    debug!("Finished setting up server-side TLS config, attempting to create HTTPS client.");

    let client = Client::new(client_config.ca_location.as_ref())
        .map_err(|why| format!("Could not create https client: {:?}", why))?;

    debug!("Created HTTPS client, attempting to bind {}.", &server_config.addr);

    let listener = TcpListener::bind(&server_config.addr).await
        .map_err(|why| format!("Could not bind {}: {:?}", &server_config.addr, why))?;

    info!("https-proxy has started. Listening for new connections on {:?}.", &server_config.addr);

    Ok(
        (client, listener, tls_acceptor, mirrors)
    )
}

fn create_logger() -> Result<(), SetLoggerError> {
    let colors = ColoredLevelConfig::new()
        .trace(Color::White)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                chrono::Local::now().format("%I:%M:%S%.3f %p"),
                colors.color(record.level()),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to initiate CTRL+C signal handler.");
}