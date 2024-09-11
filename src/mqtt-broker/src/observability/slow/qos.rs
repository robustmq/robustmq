use std::sync::Arc;

use common_base::tools::{get_local_ip, now_mills};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::handler::cache::CacheManager;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SlowMessage {
    client_id: String,
    topic: String,
    time_ms: u16,
    node_info: String,
    create_time: u128,
}

pub fn try_record_slow_message(
    _: Arc<CacheManager>,
    client_id: String,
    topic: String,
    time_ms: u16,
) {
    let ip = get_local_ip();
    let slow = SlowMessage {
        client_id,
        topic,
        time_ms,
        node_info: format!("RobustMQ-MQTT@{}", ip),
        create_time: now_mills(),
    };

    match serde_json::to_string(&slow) {
        Ok(data) => info!("{}", data),
        Err(e) => error!(
            "Failed to serialize slow subscribe message with error message :{}",
            e.to_string()
        ),
    }
}
