use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct StorageDataStructBroker {
    pub node_id: u64,
    pub addr: String,
}
