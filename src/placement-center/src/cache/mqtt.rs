use dashmap::DashMap;
use metadata_struct::mqtt::{session::MQTTSession, topic::MQTTTopic, user::MQTTUser};

pub struct MqttCacheManager {
    pub topic_list: DashMap<String, MQTTTopic>,
    pub session_list: DashMap<String, MQTTSession>,
    pub user_list: DashMap<String, MQTTUser>,
}

impl MqttCacheManager {
    pub fn new() -> MqttCacheManager {
        return MqttCacheManager {
            topic_list: DashMap::with_capacity(8),
            user_list: DashMap::with_capacity(8),
            session_list: DashMap::with_capacity(256),
        };
    }

    pub fn load_cache(){
        
    }
}
