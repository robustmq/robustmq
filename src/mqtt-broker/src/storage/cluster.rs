use std::sync::Arc;

use super::keys::cluster_config_key;
use crate::metadata::cluster::Cluster;
use common_base::errors::RobustMQError;
use storage_adapter::{memory::MemoryStorageAdapter, storage::StorageAdapter};

pub struct ClusterStorage {
    storage_adapter: Arc<MemoryStorageAdapter>,
}

impl ClusterStorage {
    pub fn new(storage_adapter: Arc<MemoryStorageAdapter>) -> Self {
        return ClusterStorage { storage_adapter };
    }

    // Get session information for the connection dimension
    pub async fn get_cluster_config(&self) -> Result<Option<Cluster>, RobustMQError> {
        let key = cluster_config_key();
        match self.storage_adapter.kv_get(key).await {
            Ok(data) => {
                if let Some(message) = data {
                    match serde_json::from_slice(&message.data) {
                        Ok(da) => {
                            return Ok(da);
                        }
                        Err(e) => {
                            return Err(common_base::errors::RobustMQError::CommmonError(
                                e.to_string(),
                            ))
                        }
                    }
                }
                return Ok(None);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
