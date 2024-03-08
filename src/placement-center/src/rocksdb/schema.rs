use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum StorageDataType {
    RegisterNode,
    UngisterNode,
    CreateShard,
    DeleteShard,
}

impl fmt::Display for StorageDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageDataType::RegisterNode => {
                write!(f, "RegisterNode")
            }
            StorageDataType::UngisterNode => {
                write!(f, "UngisterNode")
            }
            StorageDataType::CreateShard => {
                write!(f, "CreateShard")
            }
            StorageDataType::DeleteShard => {
                write!(f, "DeleteShard")
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageData {
    pub data_type: StorageDataType,
    pub value: Vec<u8>,
}

impl StorageData {
    pub fn new(data_type: StorageDataType, value: Vec<u8>) -> StorageData {
        return StorageData {
            data_type,
            value: value,
        };
    }
}

impl fmt::Display for StorageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {:?})", self.data_type, self.value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageDataStructBroker {
    pub node_id: u64,
    pub addr: String,
}
