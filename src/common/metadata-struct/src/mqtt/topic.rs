use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MQTTTopic {
    pub topic_id: String,
    pub topic_name: String,
    pub retain_message: Option<Vec<u8>>,
    pub retain_message_expired_at: Option<u64>,
}

impl MQTTTopic {
    pub fn new(topic_id: String, topic_name: String) -> Self {
        return MQTTTopic {
            topic_id,
            topic_name: topic_name,
            retain_message: None,
            retain_message_expired_at: None,
        };
    }

    pub fn encode(&self) -> Vec<u8> {
        return serde_json::to_vec(&self).unwrap();
    }
}
