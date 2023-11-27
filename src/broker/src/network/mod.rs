use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use self::{connection::ConnectionManager, rector::Rector};
mod connection;
mod network;
mod rector;

pub struct NetworkServer {
    ip: SocketAddr,
    accept_thread_num: usize,
    max_connection_num: usize,
}

impl NetworkServer {

    pub fn new(ip: SocketAddr, accept_thread_num: usize, max_connection_num: usize) -> Self {
        Self {
            ip,
            accept_thread_num,
            max_connection_num,
        }
    }

    pub fn start(&self) {
        let connection_manager =
            Arc::new(RwLock::new(ConnectionManager::new(self.max_connection_num)));
        let rector = Rector::new(self.ip, self.accept_thread_num, connection_manager);
        rector.start();
    }
}

#[cfg(test)]
mod test {
    use std::net::SocketAddr;

    use super::NetworkServer;

    #[test]

    fn network_server_test() {
        let ip: SocketAddr = "127.0.0.1:8768".parse().unwrap();
        let net_s = NetworkServer::new(ip, 10, 10);
    }
}
