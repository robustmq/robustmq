use std::collections::HashMap;

use common_base::tools::now_mills;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct HeartbeatManager {
    pub heartbeat_data: HashMap<u64, u128>,
}

impl HeartbeatManager {
    pub fn new() -> Self {
        return HeartbeatManager {
            heartbeat_data: HashMap::new(),
        };
    }

    pub fn report_hearbeat(&mut self, connect_id: u64) {
        self.heartbeat_data.insert(connect_id, now_mills());
    }

    pub fn remove_connect(&mut self, connect_id: u64) {
        self.heartbeat_data.remove(&connect_id);
    }
}
