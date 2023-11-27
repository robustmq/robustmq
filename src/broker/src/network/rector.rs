use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use super::connection::{Connection, ConnectionManager, Error};

pub struct Rector {
    bind_addr: SocketAddr,
    nums: usize,
    connection_manager: Arc<RwLock<ConnectionManager>>,
}

impl Rector {
    pub fn new(
        addr: SocketAddr,
        nums: usize,
        connection_manager: Arc<RwLock<ConnectionManager>>,
    ) -> Rector {
        Rector {
            bind_addr: addr,
            nums,
            connection_manager,
        }
    }

    pub async fn start(&self) {
        let listener = TcpListener::bind(self.bind_addr).await.unwrap();
        let at = Acceptor::new(listener, self.nums, self.connection_manager.clone());
        at.start().await;
    }
    
}

pub struct Acceptor {
    listener: TcpListener,
    connection_manager: Arc<RwLock<ConnectionManager>>,
    nums: usize,
}

impl Acceptor {
    pub fn new(
        listener: TcpListener,
        nums: usize,
        connection_manager: Arc<RwLock<ConnectionManager>>,
    ) -> Acceptor {
        Acceptor {
            listener,
            connection_manager,
            nums,
        }
    }

    pub async fn start(&self) -> Result<(), Error> {
        for _ in 0..=self.nums {
            let (stream, addr) = self.listener.accept().await.unwrap();
            let conn = Connection::new(stream, addr);
            let mut cm = self.connection_manager.write().await;
            cm.add(conn);
            // tokio::spawn();
        }
        return Ok(());
    }
}
