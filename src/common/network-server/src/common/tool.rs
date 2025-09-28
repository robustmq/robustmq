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

use crate::common::{channel::RequestChannel, packet::RequestPackage};
use common_metrics::mqtt::packets::record_mqtt_packet_received_metrics;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::{mqtt::common::MqttPacket, robust::RobustMQPacket};
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
    match pack.clone() {
        RobustMQPacket::KAFKA(_) => {}
        RobustMQPacket::MQTT(pack) => {
            record_mqtt_packet_received_metrics(connection, &pack, network_type);
        }
    }

    let package = RequestPackage::new(connection.connection_id, connection.addr, pack);
    request_channel
        .send_request_channel(network_type, package.clone())
        .await;
}
