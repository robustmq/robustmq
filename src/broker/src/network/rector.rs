use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::{atomic::AtomicU64, Arc},
};

use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Description The number of TCP connections on a node exceeded the upper limit. The maximum number of TCP connections was {total:?}")]
    ConnectionExceed { total: usize },
}

pub struct Rector {
    bind_addr: SocketAddr,
}

impl Rector {
    pub fn new(addr: SocketAddr) -> Rector {
        Rector { bind_addr: addr }
    }

    pub async fn start(&self) -> Result<TcpListener, io::Error> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        return Ok(listener);
    }
}

pub struct Acceptor {
    listener: TcpListener,
    connection_manager: Arc<ConnectionManager>,
    nums: usize,
}

impl Acceptor {
    pub fn new(
        listener: TcpListener,
        nums: usize,
        connection_manager: Arc<ConnectionManager>,
    ) -> Acceptor {
        Acceptor {
            listener,
            connection_manager,
            nums,
        }
    }

    pub async fn start(&self) {
        let (stream, addr) = self.listener.accept().await.unwrap();
        let conn = Connection::new(stream, addr);
        // self.connection_manager.add(conn)
    }
}

pub struct ConnectionManager {
    connections: HashMap<u64, Connection>,
    max_connection_num: usize,
}

impl ConnectionManager {
    pub fn new(max_connection_num: usize) -> ConnectionManager {
        ConnectionManager {
            connections: HashMap::new(),
            max_connection_num,
        }
    }

    pub fn add(&mut self, connection: Connection) -> Result<(),Error> {
        if self.connections.capacity() >= self.max_connection_num {
            return Err(Error::ConnectionExceed { total: self.max_connection_num });
        }
        self.connections
            .insert(connection.connection_id, connection);
        Ok(())
    }

    pub fn remove(&mut self, connection_id: u64) {
        self.connections.remove(&connection_id);
    }
}

static CONNECTION_ID_BUILD: AtomicU64 = AtomicU64::new(1);
pub struct Connection {
    connection_id: u64,
    socket: TcpStream,
    addr: SocketAddr,
}

impl Connection {
    pub fn new(socket: TcpStream, addr: SocketAddr) -> Connection {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Connection {
            connection_id,
            socket,
            addr,
        }
    }
}
