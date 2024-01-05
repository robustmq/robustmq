use bincode::{deserialize, serialize};

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
            StorageDataType::RegisterBroker => {
                return self.register_broker(storage_data.data);
            }
            StorageDataType::UnRegisterBroker => {
                return self.unregister_broker(storage_data.data);
            }
        }
        return Ok(());
    }

    pub fn register_broker(&self, data: String) -> Result<(), MetaError> {
        return Ok(());
    }

    pub fn unregister_broker(&self, data: String) -> Result<(), MetaError> {
        return Ok(());
    }
}
