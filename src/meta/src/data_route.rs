use bincode::{deserialize, serialize};

use crate::{
    errors::MetaError,
    storage::{StorageData, StorageDataType},
};

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
                return self.register_broker();
            }
            StorageDataType::UnRegisterBroker => {
                return self.unregister_broker();
            }
        }
        return Ok(());
    }

    pub fn register_broker(&self) -> Result<(), MetaError> {
        return Ok(());
    }

    pub fn unregister_broker(&self) -> Result<(), MetaError> {
        return Ok(());
    }
}
