use std::sync::Arc;

use common_base::error::common::CommonError;
use metadata_struct::acl::mqtt_blacklist::MQTTBlackList;

use crate::storage::{
    engine::engine_save_by_cluster, keys::storage_key_mqtt_blacklist, rocksdb::RocksDBEngine,
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
        blacklist: MQTTBlackList,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(
            cluster_name,
            &blacklist.black_list_type,
            &blacklist.resource_name,
        );
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, blacklist);
    }

    pub fn get(
        &self,
        cluster_name: &String,
        client_id: &String,
    ) -> Result<Option<MQTTBlackList>, CommonError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key) {
            Ok(Some(data)) => match serde_json::from_slice::<LastWillData>(&data.data) {
                Ok(lastwill) => {
                    return Ok(Some(lastwill));
                }
                Err(e) => {
                    return Err(e.into());
                }
            },
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => Err(e),
        }
    }

    pub fn delete(&self, cluster_name: &String, client_id: &String) -> Result<(), CommonError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }
}
