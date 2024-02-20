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
            StorageDataType::RegisterNode => {
                return self.register_node(storage_data.value);
            }
            StorageDataType::UngisterNode => {
                return self.unregister_node(storage_data.value);
            }
        }
    }

    pub fn register_node(&self, value: Vec<u8>) -> Result<(), MetaError> {

        return Ok(());
    }

    pub fn unregister_node(&self, value: Vec<u8>) -> Result<(), MetaError> {

        return Ok(());
    }
}
