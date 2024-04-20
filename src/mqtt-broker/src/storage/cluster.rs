use std::sync::Arc;

use super::keys::cluster_config_key;
use crate::metadata::cluster::Cluster;
use common_base::errors::RobustMQError;
use storage_adapter::{placement::PlacementStorageAdapter, storage::StorageAdapter};

pub struct ClusterStorage {
    storage_adapter: Arc<PlacementStorageAdapter>,
}

impl ClusterStorage {
    pub fn new(storage_adapter: Arc<PlacementStorageAdapter>) -> Self {
        return ClusterStorage { storage_adapter };
    }

    pub async fn get_cluster_config(&self) -> Result<Option<Cluster>, RobustMQError> {
        let key = cluster_config_key();
        match self.storage_adapter.get(key).await {
            Ok(Some(data)) => match serde_json::from_slice(&data.data) {
                Ok(da) => {
                    return Ok(da);
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(
                        e.to_string(),
                    ))
                }
            },
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
