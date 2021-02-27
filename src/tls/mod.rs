use core::task::{Context, Poll};
use std::pin::Pin;
use tokio::net::{TcpStream, TcpListener};
use futures_util::stream::Stream;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use crate::tls::stream::make_acceptor_stream;

pub mod config;
mod stream;
mod loader;

pub struct HyperTlsAcceptor<'a> {
    acceptor: Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, std::io::Error>> + 'a>>,
}

impl<'a> HyperTlsAcceptor<'a> {
    pub fn new(listener: TcpListener, tls_acceptor: TlsAcceptor) -> Self {
        let acceptor = make_acceptor_stream(listener, tls_acceptor);

        Self {
            acceptor,
        }
    }
}

impl hyper::server::accept::Accept for HyperTlsAcceptor<'_> {
    type Conn = TlsStream<TcpStream>;
    type Error = std::io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        Pin::new(&mut self.acceptor).poll_next(cx)
    }
}