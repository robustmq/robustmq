use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MQTTUser {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

impl MQTTUser {
    pub fn encode(&self) -> Vec<u8> {
        return serde_json::to_vec(&self).unwrap();
    }
}
