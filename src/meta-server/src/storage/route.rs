use bincode::{deserialize, serialize};
use common::log::info_meta;

use crate::errors::MetaError;

use super::schema::{StorageData, StorageDataType};

#[derive(Clone)]
pub struct DataRoute {}

impl DataRoute {
    pub fn new() -> DataRoute {
        return DataRoute {};
    }

    pub fn route(&self, data: Vec<u8>) -> Result<(), MetaError> {
        let storage_data: StorageData = deserialize(data.as_ref()).unwrap();
        match storage_data.data_type {
            StorageDataType::Set => {
                return self.set(storage_data.key, storage_data.value);
            }
            StorageDataType::Delete => {
                return self.delete(storage_data.key);
            }
        }
    }

    pub fn set(&self, key: String, value: Option<Vec<u8>>) -> Result<(), MetaError> {
        info_meta("persisted register broker");
        return Ok(());
    }

    pub fn delete(&self, data: String) -> Result<(), MetaError> {
        info_meta("persisted unregister broker");
        return Ok(());
    }
}
