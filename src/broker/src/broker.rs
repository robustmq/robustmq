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

pub struct Broker;

impl Broker {
    pub fn new() -> Broker{
        return Broker {  }
    }
    pub async fn start(){
        let ip: SocketAddr = "127.0.0.1:8768".parse().unwrap();
        let listener = TcpListener::bind(&ip).await;
    }
}

#[cfg(test)]
mod tests{
    use common_base::runtime::create_runtime;

    use super::Broker;

    #[test]
    fn start_broker(){
        let rt = create_runtime("text", 10);
        let guard = rt.enter();
        let b = Broker::new();
        drop(guard);
    }
}