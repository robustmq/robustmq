use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum StorageDataType {
    RegisterBroker,
    UnRegisterBroker,
}

impl fmt::Display for StorageDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageDataType::RegisterBroker => {
                write!(f, "RegisterBroker")
            }
            StorageDataType::UnRegisterBroker => {
                write!(f, "UnRegisterBroker")
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageData {
    pub data_type: StorageDataType,
    pub data: String,
}

impl StorageData {
    pub fn new(data_type: StorageDataType, data: String) -> StorageData {
        return StorageData { data_type, data };
    }
}

impl fmt::Display for StorageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.data_type, self.data)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageDataStructBroker {
    pub node_id: u64,
    pub addr: String,
}

