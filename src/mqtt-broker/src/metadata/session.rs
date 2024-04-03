use protocol::mqtt::{LastWill, LastWillProperties};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub client_id: String,
    pub keep_alive: u16,
    pub last_will: bool,
    pub session_expiry_interval: u32,
}

#[derive(Serialize, Deserialize)]
pub struct LastWillData {
    pub last_will: LastWill,
    pub last_will_properties: LastWillProperties,
}
