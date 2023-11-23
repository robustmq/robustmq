use std::{net::SocketAddr, fmt::Result};

use tokio::{net::TcpListener, time::error::Elapsed, io};

#[derive(Debug, thiserror::Error)]
#[error("Acceptor error")]
pub enum Error {
    #[error("I/O {0}")]
    Io(#[from] io::Error),
    #[error("Timeout")]
    Timeout(#[from] Elapsed),
}

pub struct Broker {}

impl Broker {
    async fn start(){
        let ip: SocketAddr = "".parse().unwrap();
        let listener = TcpListener::bind(&ip).await;
    }
}
