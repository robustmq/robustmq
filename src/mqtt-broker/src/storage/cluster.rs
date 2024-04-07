use super::keys::cluster_config_key;
use crate::metadata::cluster::Cluster;
use common_base::{config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError};
use storage_adapter::{adapter::placement::PlacementStorageAdapter, storage::StorageAdapter};

pub struct ClusterStorage {
    storage_adapter: PlacementStorageAdapter,
}

impl ClusterStorage {
    pub fn new() -> Self {
        let storage_adapter = PlacementStorageAdapter::new();
        return ClusterStorage { storage_adapter };
    }

    pub fn register_cluster_info(&self) {
        let conf = broker_mqtt_conf();
    }

    pub fn unregister_cluster_info(&self) {}

    pub fn cluster_node_list(&self) {}

    fn init_node_id(&self) {}

    // Get session information for the connection dimension
    pub fn get_cluster_config(&self) -> Result<Cluster, RobustMQError> {
        let key = cluster_config_key();
        match self.storage_adapter.kv_get(key) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(da) => {
                    return Ok(da);
                }
                Err(e) => {
                    return Err(common_base::errors::RobustMQError::CommmonError(
                        e.to_string(),
                    ))
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }
}
