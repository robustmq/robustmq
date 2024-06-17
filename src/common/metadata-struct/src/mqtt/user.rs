use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MQTTUser {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

impl MQTTUser {
    pub fn encode(&self) -> String {
        return serde_json::to_string(&self).unwrap();
    }
}
