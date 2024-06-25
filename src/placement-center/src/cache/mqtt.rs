use dashmap::DashMap;
use metadata_struct::mqtt::{topic::MQTTTopic, user::MQTTUser};

pub struct MqttCacheManager {
    pub topic_list: DashMap<String, MQTTTopic>,
    pub user_list: DashMap<String, MQTTUser>,
    pub session_expire: DashMap<String, u64>,
}

impl MqttCacheManager {
    pub fn new() -> MqttCacheManager {
        return MqttCacheManager {
            topic_list: DashMap::with_capacity(8),
            user_list: DashMap::with_capacity(8),
            session_expire: DashMap::with_capacity(8),
        };
    }

    pub fn load_cache() {}
}
