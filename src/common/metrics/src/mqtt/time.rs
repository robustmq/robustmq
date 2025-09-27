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

use crate::{histogram_metric_observe, register_histogram_metric_ms_with_default_buckets};
use metadata_struct::connection::NetworkConnectionType;
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct NetworkLabel {
    pub network: String,
    pub packet: String,
}

register_histogram_metric_ms_with_default_buckets!(
    MQTT_PACKET_PROCESS_DURATION_MS,
    "mqtt_packet_process_duration_ms",
    "MQTT packet processing duration in milliseconds",
    NetworkLabel
);

register_histogram_metric_ms_with_default_buckets!(
    MQTT_PACKET_SEND_DURATION_MS,
    "mqtt_packet_send_duration_ms",
    "MQTT packet sending duration in milliseconds",
    NetworkLabel
);

pub fn record_mqtt_packet_process_duration(
    network_type: NetworkConnectionType,
    packet: String,
    duration_ms: f64,
) {
    let label = NetworkLabel {
        network: network_type.to_string(),
        packet,
    };
    histogram_metric_observe!(MQTT_PACKET_PROCESS_DURATION_MS, duration_ms, label);
}

pub fn record_mqtt_packet_send_duration(
    network_type: NetworkConnectionType,
    packet: String,
    duration_ms: f64,
) {
    let label = NetworkLabel {
        network: network_type.to_string(),
        packet,
    };
    histogram_metric_observe!(MQTT_PACKET_SEND_DURATION_MS, duration_ms, label);
}
