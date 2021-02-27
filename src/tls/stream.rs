use std::pin::Pin;
use log::{error, debug};
use async_stream::stream;
use futures_util::stream::Stream;
use tokio_rustls::server::TlsStream;
use tokio::net::TcpStream;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

pub fn make_acceptor_stream(listener: TcpListener, acceptor: TlsAcceptor) -> Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, std::io::Error>>>> {
    let stream = stream! {
        loop {
            let (socket, addr) = listener.accept().await?;

            match acceptor.accept(socket).await {
                Ok(stream) => {
                    debug!("Accepted new TLS connection from: {:?}", addr);
                    yield Ok(stream)
                },
                Err(why) => {
                    error!("Could not accept TLS connection from {:?}: {:?}", addr, why);
                }
            }
        }
    };

    Box::pin(stream)
}