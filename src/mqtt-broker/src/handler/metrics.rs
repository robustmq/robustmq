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

use common_base::tools::now_mills;
use common_metrics::mqtt::{
    packets::record_packet_send_metrics,
    publish::{
        record_mqtt_message_bytes_received, record_mqtt_message_bytes_sent,
        record_mqtt_messages_received_inc, record_mqtt_messages_sent_inc,
    },
    session::{
        record_connection_messages_in, record_connection_messages_out, record_session_messages_in,
        record_session_messages_out,
    },
    time::record_packet_send_duration,
    topic::{
        record_topic_bytes_sent, record_topic_bytes_written, record_topic_messages_sent,
        record_topic_messages_written,
    },
};
use metadata_struct::connection::NetworkConnectionType;
use protocol::mqtt::{
    codec::MqttPacketWrapper,
    common::{mqtt_packet_to_string, MqttPacket},
};

pub fn record_publish_receive_metrics(
    client_id: &str,
    connection_id: u64,
    topic_name: &str,
    payload_len: u64,
) {
    record_mqtt_messages_received_inc();
    record_mqtt_message_bytes_received(payload_len);

    record_topic_messages_written(topic_name);
    record_topic_bytes_written(topic_name, payload_len);

    record_session_messages_in(client_id);
    record_connection_messages_in(connection_id);
}

pub fn record_publish_send_metrics(
    client_id: &str,
    connection_id: u64,
    topic_name: &str,
    payload_len: u64,
) {
    record_mqtt_messages_sent_inc();
    record_mqtt_message_bytes_sent(payload_len);
    record_topic_messages_sent(topic_name);
    record_topic_bytes_sent(topic_name, payload_len);

    record_session_messages_out(client_id);
    record_connection_messages_out(connection_id);
}

pub fn record_send_metrics(
    wrapper: &MqttPacketWrapper,
    packet: &MqttPacket,
    network_type: Option<NetworkConnectionType>,
    start: u128,
) {
    if let Some(network) = network_type.clone() {
        record_packet_send_duration(
            network,
            mqtt_packet_to_string(packet),
            (now_mills() - start) as f64,
        );
    }

    record_packet_send_metrics(wrapper, format!("{:?}", network_type));
}
