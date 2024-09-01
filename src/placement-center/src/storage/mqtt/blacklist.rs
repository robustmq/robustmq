use common_base::error::common::CommonError;
use metadata_struct::acl::mqtt_blacklist::MQTTAclBlackList;
use std::sync::Arc;

use crate::storage::{
    engine::{engine_delete_by_cluster, engine_prefix_list_by_cluster, engine_save_by_cluster},
    keys::{storage_key_mqtt_blacklist, storage_key_mqtt_blacklist_prefix},
    rocksdb::RocksDBEngine,
};

pub struct MQTTBlackListStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MQTTBlackListStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MQTTBlackListStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(
        &self,
        cluster_name: &String,
        blacklist: MQTTAclBlackList,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(
            cluster_name,
            &blacklist.blacklist_type.to_string(),
            &blacklist.resource_name,
        );
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, blacklist);
    }

    pub fn list(&self, cluster_name: &String) -> Result<Vec<MQTTAclBlackList>, CommonError> {
        let prefix_key = storage_key_mqtt_blacklist_prefix(cluster_name);
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<MQTTAclBlackList>(&raw.data) {
                        Ok(blacklist) => {
                            results.push(blacklist);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                return Ok(results);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn delete(
        &self,
        cluster_name: &String,
        blacklist_type: &String,
        resource_name: &String,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(cluster_name, blacklist_type, resource_name);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }
}
