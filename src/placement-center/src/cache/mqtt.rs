use crate::storage::{mqtt::session::MQTTSessionStorage, rocksdb::RocksDBEngine};
use dashmap::DashMap;
use metadata_struct::mqtt::{session::MQTTSession, topic::MQTTTopic, user::MQTTUser};
use std::sync::Arc;

use super::placement::PlacementCacheManager;

pub struct MqttCacheManager {
    pub topic_list: DashMap<String, MQTTTopic>,
    pub user_list: DashMap<String, MQTTUser>,
    pub last_will_message: DashMap<String, u64>,
}

impl MqttCacheManager {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_cache: Arc<PlacementCacheManager>,
    ) -> MqttCacheManager {
        return MqttCacheManager {
            topic_list: DashMap::with_capacity(8),
            user_list: DashMap::with_capacity(8),
            last_will_message: DashMap::with_capacity(8),
        };
    }

    pub fn load_cache(
        &self,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_cache: Arc<PlacementCacheManager>,
    ) {
        
        
    }
}
