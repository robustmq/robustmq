use std::sync::Arc;

use clients::ClientPool;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct MySQLStorageAdapter {
    addrs: String,
}

impl MySQLStorageAdapter {
    pub fn new(client_poll: Arc<Mutex<ClientPool>>, addrs: Vec<String>) -> Self {
        return MySQLStorageAdapter { client_poll, addrs };
    }
}
