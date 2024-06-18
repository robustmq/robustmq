use common_base::tools::unique_id;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MQTTTopic {
    pub topic_id: String,
    pub topic_name: String,
    pub retain_message: Option<Vec<u8>>,
}

impl MQTTTopic {
    pub fn new(topic_name: &String) -> Self {
        return MQTTTopic {
            topic_id: unique_id(),
            topic_name: topic_name.clone(),
            retain_message: None,
        };
    }

    pub fn encode(&self) -> String {
        return serde_json::to_string(&self).unwrap();
    }
}
