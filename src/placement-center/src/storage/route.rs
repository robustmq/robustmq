use std::sync::{Arc, RwLock};

use bincode::{deserialize, serialize};
use common::log::info_meta;

use crate::errors::MetaError;

use super::{
    cluster_storage::ClusterStorage,
    schema::{StorageData, StorageDataType},
};

#[derive(Clone)]
pub struct DataRoute {
    cluster_storage: Arc<RwLock<ClusterStorage>>,
}

impl DataRoute {
    pub fn new(cluster_storage: Arc<RwLock<ClusterStorage>>) -> DataRoute {
        return DataRoute { cluster_storage };
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
        let storage = self.cluster_storage.write().unwrap();
        storage.set(key, value.unwrap());
        return Ok(());
    }

    pub fn delete(&self, key: String) -> Result<(), MetaError> {
        let storage = self.cluster_storage.write().unwrap();
        storage.delete(key);
        info_meta("persisted unregister broker");
        return Ok(());
    }
}
