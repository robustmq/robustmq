use std::sync::Arc;

use common_base::tools::now_mills;
use log::{error, info};
use protocol::mqtt::codec::parse_mqtt_packet_to_name;
use serde::{Deserialize, Serialize};

use crate::{handler::cache::CacheManager, server::packet::RequestPackage};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SlowRequestMs {
    command: String,
    time_ms: u128,
}

// total processing time of the request packet was recorded
pub fn try_record_total_request_ms(cache_manager: Arc<CacheManager>, package: RequestPackage) {
    
    let cluster_config = cache_manager.get_cluster_info();
    if !cluster_config.slow.enable {
        return;
    }

    let whole = cluster_config.slow.whole_ms;
    let time_ms = now_mills() - package.receive_ms;
    if time_ms < whole {
        return;
    }

    let command = parse_mqtt_packet_to_name(package.packet);
    let slow = SlowRequestMs { command, time_ms };

    match serde_json::to_string(&slow) {
        Ok(data) => info!("{}", data),
        Err(e) => error!(
            "Failed to serialize slow subscribe message with error message :{}",
            e.to_string()
        ),
    }
}
