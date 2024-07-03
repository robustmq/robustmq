use crate::{
    controller::mqtt::session_expire::ExpireLastWill,
    storage::{placement::cluster::ClusterStorage, rocksdb::RocksDBEngine},
};
use dashmap::DashMap;
use metadata_struct::mqtt::{topic::MQTTTopic, user::MQTTUser};
use protocol::placement_center::generate::common::ClusterType;
use std::sync::Arc;

use super::placement::PlacementCacheManager;

pub struct MqttCacheManager {
    pub topic_list: DashMap<String, MQTTTopic>,
    pub user_list: DashMap<String, MQTTUser>,
    pub expire_last_wills: DashMap<String, ExpireLastWill>,
}

impl MqttCacheManager {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_cache: Arc<PlacementCacheManager>,
    ) -> MqttCacheManager {
        return MqttCacheManager {
            topic_list: DashMap::with_capacity(8),
            user_list: DashMap::with_capacity(8),
            expire_last_wills: DashMap::with_capacity(8),
        };
    }

    pub fn add_expire_last_will(&self, expire_last_will: ExpireLastWill) {
        let key =
            self.expire_last_will_key(&expire_last_will.cluster_name, &expire_last_will.client_id);
        self.expire_last_wills.insert(key, expire_last_will);
    }

    pub fn remove_expire_last_will(&self, cluster_name: &String, client_id: &String) {
        let key = self.expire_last_will_key(cluster_name, client_id);
        self.expire_last_wills.remove(&key);
    }

    fn expire_last_will_key(&self, cluster_name: &String, client_id: &String) -> String {
        return format!("{}_{}", cluster_name, client_id);
    }
    pub fn load_cache(
        &self,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_cache: Arc<PlacementCacheManager>,
    ) {
        
    }
}
