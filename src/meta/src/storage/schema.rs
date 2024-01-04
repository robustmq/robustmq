use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum StorageDataType {
    RegisterBroker,
    UnRegisterBroker,
}

#[derive(Deserialize, Serialize)]
pub struct StorageData {
    pub data_type: StorageDataType,
    pub data: Vec<u8>,
}

impl StorageData {
    pub fn new(data_type: StorageDataType, data: Vec<u8>) -> StorageData {
        return StorageData { data_type, data };
    }
}

#[derive(Deserialize, Serialize)]
pub struct StorageDataStructBroker {
    pub node_id: u64,
    pub addr: String,
}
