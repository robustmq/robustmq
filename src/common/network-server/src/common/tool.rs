// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use crate::common::{
    channel::RequestChannel, connection_manager::ConnectionManager, packet::RequestPackage,
};
use broker_core::cache::NodeCacheManager;
use common_base::error::common::CommonError;
use common_metrics::mqtt::packets::record_packet_received_metrics;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::{mqtt::common::MqttPacket, robust::RobustMQPacket};
use rate_limit::global::GlobalRateLimiterManager;
use tracing::debug;

pub fn is_ignore_print(packet: &RobustMQPacket) -> bool {
    if let RobustMQPacket::MQTT(pack) = packet {
        if let MqttPacket::PingResp(_) = pack {
            return true;
        }
        if let MqttPacket::PingReq(_) = pack {
            return true;
        }
    }

    false
}

pub async fn read_packet(
    pack: RobustMQPacket,
    request_channel: &RequestChannel,
    connection: &NetworkConnection,
    network_type: &NetworkConnectionType,
) {
    if !is_ignore_print(&pack) {
        debug!(
            "recv {} packet:{:?}, connect_id:{}",
            network_type, pack, connection.connection_id
        );
    }
    if let RobustMQPacket::MQTT(mqtt_pack) = &pack {
        record_packet_received_metrics(connection, mqtt_pack, network_type);
    }

    let package = RequestPackage::new(
        connection.connection_id,
        connection.addr,
        pack,
        network_type.clone(),
    );
    request_channel.send(package).await;
}

pub async fn check_connection_limit(
    global_limit_manager: &Arc<GlobalRateLimiterManager>,
    node_cache: &Arc<NodeCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<bool, CommonError> {
    // connection rate limit
    global_limit_manager.network_connection_rate_limit().await?;

    // connection count limit
    let limit = node_cache.get_cluster_config().cluster_limit;
    if connection_manager.connections.len() > limit.max_network_connection as usize {
        return Ok(true);
    }
    Ok(false)
}
