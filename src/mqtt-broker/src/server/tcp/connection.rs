use std::{net::SocketAddr, sync::atomic::AtomicU64};

static CONNECTION_ID_BUILD: AtomicU64 = AtomicU64::new(1);

pub struct Connection {
    pub connection_id: u64,
    pub addr: SocketAddr,
}

impl Connection {
    pub fn new(addr: SocketAddr) -> Self {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Connection {
            connection_id,
            addr,
        }
    }

    pub fn connection_id(&self) -> u64 {
        return self.connection_id;
    }
}
