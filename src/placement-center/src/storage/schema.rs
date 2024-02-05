use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum StorageDataType {
    Set,
    Delete,
}

impl fmt::Display for StorageDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageDataType::Set => {
                write!(f, "RegisterBroker")
            }
            StorageDataType::Delete => {
                write!(f, "UnRegisterBroker")
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageData {
    pub data_type: StorageDataType,
    pub key: String,
    pub value: Option<Vec<u8>>,
}

impl StorageData {
    pub fn new(data_type: StorageDataType, key: String, value: Vec<u8>) -> StorageData {
        return StorageData {
            data_type,
            key,
            value: Some(value),
        };
    }
}

impl fmt::Display for StorageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {:?})", self.data_type, self.key, self.value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageDataStructBroker {
    pub node_id: u64,
    pub addr: String,
}
