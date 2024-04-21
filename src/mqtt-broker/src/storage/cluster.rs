use super::keys::cluster_config_key;
use crate::metadata::cluster::Cluster;
use common_base::errors::RobustMQError;
use std::sync::Arc;
use storage_adapter::{record::Record, storage::StorageAdapter};

pub struct ClusterStorage<T> {
    storage_adapter: Arc<T>,
}

impl<T> ClusterStorage<T>
where
    T: StorageAdapter,
{
    pub fn new(storage_adapter: Arc<T>) -> Self {
        return ClusterStorage { storage_adapter };
    }

    // Saving user information
    pub async fn save_cluster(&self, cluster: Cluster) -> Result<(), RobustMQError> {
        let key = cluster_config_key();
        match serde_json::to_vec(&cluster) {
            Ok(data) => {
                return self.storage_adapter.set(key, Record::build_b(data)).await;
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(format!(
                    "save cluster config error, error messsage:{}",
                    e.to_string()
                )))
            }
        }
    }

    pub async fn get_cluster_config(&self) -> Result<Option<Cluster>, RobustMQError> {
        let key = cluster_config_key();
        match self.storage_adapter.get(key).await {
            Ok(Some(data)) => match serde_json::from_slice(&data.data) {
                Ok(da) => {
                    return Ok(da);
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(format!(
                        "get cluster config error, error messsage:{}",
                        e.to_string()
                    )))
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
