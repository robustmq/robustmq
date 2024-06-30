use protocol::mqtt::common::{LastWill, LastWillProperties};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LastWillData {
    pub client_id:String,
    pub last_will: Option<LastWill>,
    pub last_will_properties: Option<LastWillProperties>,
}

impl LastWillData {
    pub fn encode(&self) -> Vec<u8> {
        return serde_json::to_vec(&self).unwrap();
    }
}
