use std::{collections::HashMap, net::SocketAddr, sync::atomic::AtomicU64};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Description The number of TCP connections on a node exceeded the upper limit. The maximum number of TCP connections was {total:?}")]
    ConnectionExceed { total: usize },
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

    pub fn add(&mut self, connection: Connection) -> Result<(), Error> {
        if self.connections.capacity() >= self.max_connection_num {
            return Err(Error::ConnectionExceed {
                total: self.max_connection_num,
            });
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
    addr: SocketAddr,
}

impl Connection {
    pub fn new(addr: SocketAddr) -> Connection {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Connection {
            connection_id,
            addr,
        }
    }
}
