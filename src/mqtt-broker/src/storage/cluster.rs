use super::keys::cluster_config_key;
use crate::metadata::cluster::Cluster;
use common_base::errors::RobustMQError;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

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
