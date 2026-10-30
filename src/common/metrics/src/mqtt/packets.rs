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

use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use prometheus_client::encoding::EncodeLabelSet;
use protocol::mqtt::{
    codec::{calc_mqtt_packet_size, MqttPacketWrapper},
    common::{ConnectReturnCode, MqttPacket, QoS},
};

use crate::{
    counter_metric_get, counter_metric_inc, counter_metric_inc_by, counter_metric_touch,
    register_counter_metric,
};

const ALL_NETWORK_TYPES: &[NetworkConnectionType] = &[
    NetworkConnectionType::Tcp,
    NetworkConnectionType::Tls,
    NetworkConnectionType::WebSocket,
    NetworkConnectionType::WebSockets,
    NetworkConnectionType::QUIC,
];

// Empty label for total (cross-network) counters
#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct TotalLabel {}

register_counter_metric!(
    MQTT_TOTAL_BYTES_RECEIVED,
    "mqtt_total_bytes_received",
    "Total MQTT bytes received across all network types",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_BYTES_SENT,
    "mqtt_total_bytes_sent",
    "Total MQTT bytes sent across all network types",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_RECEIVED,
    "mqtt_total_packets_received",
    "Total MQTT packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_SENT,
    "mqtt_total_packets_sent",
    "Total MQTT packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_CONNECT,
    "mqtt_total_packets_connect",
    "Total MQTT CONNECT packets",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_CONNACK,
    "mqtt_total_packets_connack",
    "Total MQTT CONNACK packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PUBLISH_RECEIVED,
    "mqtt_total_packets_publish_received",
    "Total MQTT PUBLISH packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PUBLISH_SENT,
    "mqtt_total_packets_publish_sent",
    "Total MQTT PUBLISH packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PUBACK_RECEIVED,
    "mqtt_total_packets_puback_received",
    "Total MQTT PUBACK packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PUBACK_SENT,
    "mqtt_total_packets_puback_sent",
    "Total MQTT PUBACK packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PUBREC_RECEIVED,
    "mqtt_total_packets_pubrec_received",
    "Total MQTT PUBREC packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PUBREC_SENT,
    "mqtt_total_packets_pubrec_sent",
    "Total MQTT PUBREC packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PUBREL_RECEIVED,
    "mqtt_total_packets_pubrel_received",
    "Total MQTT PUBREL packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PUBREL_SENT,
    "mqtt_total_packets_pubrel_sent",
    "Total MQTT PUBREL packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PUBCOMP_RECEIVED,
    "mqtt_total_packets_pubcomp_received",
    "Total MQTT PUBCOMP packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PUBCOMP_SENT,
    "mqtt_total_packets_pubcomp_sent",
    "Total MQTT PUBCOMP packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_SUBSCRIBE,
    "mqtt_total_packets_subscribe",
    "Total MQTT SUBSCRIBE packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_SUBACK,
    "mqtt_total_packets_suback",
    "Total MQTT SUBACK packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_UNSUBSCRIBE,
    "mqtt_total_packets_unsubscribe",
    "Total MQTT UNSUBSCRIBE packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_UNSUBACK,
    "mqtt_total_packets_unsuback",
    "Total MQTT UNSUBACK packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PINGREQ,
    "mqtt_total_packets_pingreq",
    "Total MQTT PINGREQ packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_PINGRESP,
    "mqtt_total_packets_pingresp",
    "Total MQTT PINGRESP packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_DISCONNECT_RECEIVED,
    "mqtt_total_packets_disconnect_received",
    "Total MQTT DISCONNECT packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_DISCONNECT_SENT,
    "mqtt_total_packets_disconnect_sent",
    "Total MQTT DISCONNECT packets sent",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_AUTH,
    "mqtt_total_packets_auth",
    "Total MQTT AUTH packets received",
    TotalLabel
);
register_counter_metric!(
    MQTT_TOTAL_PACKETS_AUTH_SENT,
    "mqtt_total_packets_auth_sent",
    "Total MQTT AUTH packets sent",
    TotalLabel
);

macro_rules! total_get {
    ($metric:ident) => {{
        let label = TotalLabel {};
        let mut result = 0u64;
        counter_metric_get!($metric, label, result);
        result
    }};
}

pub fn record_mqtt_total_bytes_received_get() -> u64 {
    total_get!(MQTT_TOTAL_BYTES_RECEIVED)
}
pub fn record_mqtt_total_bytes_sent_get() -> u64 {
    total_get!(MQTT_TOTAL_BYTES_SENT)
}
pub fn record_mqtt_total_packets_received_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_RECEIVED)
}
pub fn record_mqtt_total_packets_sent_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_SENT)
}
pub fn record_mqtt_total_packets_connect_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_CONNECT)
}
pub fn record_mqtt_total_packets_connack_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_CONNACK)
}
pub fn record_mqtt_total_packets_publish_received_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PUBLISH_RECEIVED)
}
pub fn record_mqtt_total_packets_publish_sent_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PUBLISH_SENT)
}
pub fn record_mqtt_total_packets_puback_received_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PUBACK_RECEIVED)
}
pub fn record_mqtt_total_packets_puback_sent_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PUBACK_SENT)
}
pub fn record_mqtt_total_packets_pubrec_received_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PUBREC_RECEIVED)
}
pub fn record_mqtt_total_packets_pubrec_sent_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PUBREC_SENT)
}
pub fn record_mqtt_total_packets_pubrel_received_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PUBREL_RECEIVED)
}
pub fn record_mqtt_total_packets_pubrel_sent_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PUBREL_SENT)
}
pub fn record_mqtt_total_packets_pubcomp_received_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PUBCOMP_RECEIVED)
}
pub fn record_mqtt_total_packets_pubcomp_sent_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PUBCOMP_SENT)
}
pub fn record_mqtt_total_packets_subscribe_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_SUBSCRIBE)
}
pub fn record_mqtt_total_packets_suback_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_SUBACK)
}
pub fn record_mqtt_total_packets_unsubscribe_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_UNSUBSCRIBE)
}
pub fn record_mqtt_total_packets_unsuback_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_UNSUBACK)
}
pub fn record_mqtt_total_packets_pingreq_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PINGREQ)
}
pub fn record_mqtt_total_packets_pingresp_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_PINGRESP)
}
pub fn record_mqtt_total_packets_disconnect_received_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_DISCONNECT_RECEIVED)
}
pub fn record_mqtt_total_packets_disconnect_sent_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_DISCONNECT_SENT)
}
pub fn record_mqtt_total_packets_auth_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_AUTH)
}
pub fn record_mqtt_total_packets_auth_sent_get() -> u64 {
    total_get!(MQTT_TOTAL_PACKETS_AUTH_SENT)
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct NetworkLabel {
    pub network: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct NetworkQosLabel {
    pub network: String,
    pub qos: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct QosLabel {
    pub qos: String,
}

register_counter_metric!(
    MQTT_PACKETS_RECEIVED,
    "mqtt_packets_received",
    "Number of MQTT packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_CONNECT_RECEIVED,
    "mqtt_packets_connect_received",
    "Number of MQTT CONNECT packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_PUBLISH_RECEIVED,
    "mqtt_packets_publish_received",
    "Number of MQTT PUBLISH packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_CONNACK_RECEIVED,
    "mqtt_packets_connack_received",
    "Number of MQTT CONNACK packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_PUBACK_RECEIVED,
    "mqtt_packets_puback_received",
    "Number of MQTT PUBACK packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_PUBREC_RECEIVED,
    "mqtt_packets_pubrec_received",
    "Number of MQTT PUBREC packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_PUBREL_RECEIVED,
    "mqtt_packets_pubrel_received",
    "Number of MQTT PUBREL packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_PUBCOMP_RECEIVED,
    "mqtt_packets_pubcomp_received",
    "Number of MQTT PUBCOMP packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_SUBSCRIBE_RECEIVED,
    "mqtt_packets_subscribe_received",
    "Number of MQTT SUBSCRIBE packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_UNSUBSCRIBE_RECEIVED,
    "mqtt_packets_unsubscribe_received",
    "Number of MQTT UNSUBSCRIBE packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_PINGREQ_RECEIVED,
    "mqtt_packets_pingreq_received",
    "Number of MQTT PINGREQ packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_DISCONNECT_RECEIVED,
    "mqtt_packets_disconnect_received",
    "Number of MQTT DISCONNECT packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_AUTH_RECEIVED,
    "mqtt_packets_auth_received",
    "Number of MQTT AUTH packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_RECEIVED_ERROR,
    "mqtt_packets_received_error",
    "Number of MQTT error packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_CONNACK_AUTH_ERROR,
    "mqtt_packets_connack_auth_error",
    "Number of MQTT CONNACK auth error packets received",
    NetworkLabel
);
register_counter_metric!(
    MQTT_PACKETS_CONNACK_ERROR,
    "mqtt_packets_connack_error",
    "Number of MQTT CONNACK error packets received",
    NetworkLabel
);

register_counter_metric!(
    MQTT_PACKETS_SENT,
    "mqtt_packets_sent",
    "Number of MQTT packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_CONNACK_SENT,
    "mqtt_packets_connack_sent",
    "Number of MQTT CONNACK packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_PUBLISH_SENT,
    "mqtt_packets_publish_sent",
    "Number of MQTT PUBLISH packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_PUBACK_SENT,
    "mqtt_packets_puback_sent",
    "Number of MQTT PUBACK packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_PUBREC_SENT,
    "mqtt_packets_pubrec_sent",
    "Number of MQTT PUBREC packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_PUBREL_SENT,
    "mqtt_packets_pubrel_sent",
    "Number of MQTT PUBREL packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_PUBCOMP_SENT,
    "mqtt_packets_pubcomp_sent",
    "Number of MQTT PUBCOMP packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_SUBACK_SENT,
    "mqtt_packets_suback_sent",
    "Number of MQTT SUBACK packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_UNSUBACK_SENT,
    "mqtt_packets_unsuback_sent",
    "Number of MQTT UNSUBACK packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_PINGRESP_SENT,
    "mqtt_packets_pingresp_sent",
    "Number of MQTT PINGRESP packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_DISCONNECT_SENT,
    "mqtt_packets_disconnect_sent",
    "Number of MQTT DISCONNECT packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_PACKETS_AUTH_SENT,
    "mqtt_packets_auth_sent",
    "Number of MQTT AUTH packets sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_BYTES_RECEIVED,
    "mqtt_bytes_received",
    "Number of MQTT bytes received",
    NetworkLabel
);

register_counter_metric!(
    MQTT_BYTES_SENT,
    "mqtt_bytes_sent",
    "Number of MQTT bytes sent",
    NetworkQosLabel
);

register_counter_metric!(
    MQTT_RETAIN_PACKETS_RECEIVED,
    "mqtt_retain_packets_received",
    "Number of MQTT retain messages received",
    QosLabel
);

register_counter_metric!(
    MQTT_RETAIN_PACKETS_SENT,
    "mqtt_retain_packets_sent",
    "Number of MQTT retain messages sent",
    QosLabel
);

// Record the packet-related metrics received by the server for failed resolution
pub fn record_received_error_metrics(network_type: NetworkConnectionType) {
    let labe = NetworkLabel {
        network: network_type.to_string(),
    };
    counter_metric_inc!(MQTT_PACKETS_RECEIVED_ERROR, labe);
}

// Record metrics related to packets received by the server
pub fn record_packet_received_metrics(
    connection: &NetworkConnection,
    pkg: &MqttPacket,
    network_type: &NetworkConnectionType,
) {
    let label = NetworkLabel {
        network: network_type.to_string(),
    };
    let payload_size = if let Some(protocol) = connection.protocol.clone() {
        let wrapper = MqttPacketWrapper {
            protocol_version: protocol.to_u8(),
            packet: pkg.clone(),
        };
        calc_mqtt_packet_size(wrapper)
    } else {
        0
    };

    let total = TotalLabel {};
    counter_metric_inc_by!(MQTT_BYTES_RECEIVED, label, payload_size as u64);
    counter_metric_inc_by!(MQTT_TOTAL_BYTES_RECEIVED, total, payload_size as u64);
    counter_metric_inc!(MQTT_PACKETS_RECEIVED, label);
    counter_metric_inc!(MQTT_TOTAL_PACKETS_RECEIVED, total);

    match pkg {
        MqttPacket::Connect(_, _, _, _, _, _) => {
            counter_metric_inc!(MQTT_PACKETS_CONNECT_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_CONNECT, total);
        }
        MqttPacket::ConnAck(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_CONNACK_RECEIVED, label)
        }
        MqttPacket::Publish(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_PUBLISH_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PUBLISH_RECEIVED, total);
        }
        MqttPacket::PubAck(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_PUBACK_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PUBACK_RECEIVED, total);
        }
        MqttPacket::PubRec(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_PUBREC_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PUBREC_RECEIVED, total);
        }
        MqttPacket::PubRel(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_PUBREL_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PUBREL_RECEIVED, total);
        }
        MqttPacket::PubComp(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_PUBCOMP_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PUBCOMP_RECEIVED, total);
        }
        MqttPacket::PingReq(_) => {
            counter_metric_inc!(MQTT_PACKETS_PINGREQ_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PINGREQ, total);
        }
        MqttPacket::Disconnect(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_DISCONNECT_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_DISCONNECT_RECEIVED, total);
        }
        MqttPacket::Auth(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_AUTH_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_AUTH, total);
        }
        MqttPacket::Subscribe(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_SUBSCRIBE_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_SUBSCRIBE, total);
        }
        MqttPacket::Unsubscribe(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_UNSUBSCRIBE_RECEIVED, label);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_UNSUBSCRIBE, total);
        }
        _ => unreachable!("This branch only matches for packets could not be received"),
    }
}

// Record metrics related to messages pushed to the client
pub fn record_packet_send_metrics(packet_wrapper: &MqttPacketWrapper, network_type: String) {
    let qos_str = if let MqttPacket::Publish(publish, _) = packet_wrapper.packet.clone() {
        format!("{}", publish.qos as u8)
    } else {
        "-1".to_string()
    };
    let label_qos = NetworkQosLabel {
        network: network_type.clone(),
        qos: qos_str,
    };
    let label = NetworkLabel {
        network: network_type,
    };
    let payload_size = calc_mqtt_packet_size(packet_wrapper.to_owned());
    let total = TotalLabel {};
    counter_metric_inc!(MQTT_PACKETS_SENT, label_qos);
    counter_metric_inc!(MQTT_TOTAL_PACKETS_SENT, total);
    counter_metric_inc_by!(MQTT_BYTES_SENT, label_qos, payload_size as u64);
    counter_metric_inc_by!(MQTT_TOTAL_BYTES_SENT, total, payload_size as u64);

    match &packet_wrapper.packet {
        MqttPacket::ConnAck(conn_ack, _) => {
            counter_metric_inc!(MQTT_PACKETS_CONNACK_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_CONNACK, total);
            if conn_ack.code == ConnectReturnCode::NotAuthorized {
                counter_metric_inc!(MQTT_PACKETS_CONNACK_AUTH_ERROR, label);
            }
            if conn_ack.code != ConnectReturnCode::Success {
                counter_metric_inc!(MQTT_PACKETS_CONNACK_ERROR, label);
            }
        }
        MqttPacket::Publish(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_PUBLISH_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PUBLISH_SENT, total);
        }
        MqttPacket::PubAck(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_PUBACK_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PUBACK_SENT, total);
        }
        MqttPacket::PubRec(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_PUBREC_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PUBREC_SENT, total);
        }
        MqttPacket::PubRel(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_PUBREL_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PUBREL_SENT, total);
        }
        MqttPacket::PubComp(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_PUBCOMP_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PUBCOMP_SENT, total);
        }
        MqttPacket::SubAck(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_SUBACK_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_SUBACK, total);
        }
        MqttPacket::UnsubAck(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_UNSUBACK_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_UNSUBACK, total);
        }
        MqttPacket::PingResp(_) => {
            counter_metric_inc!(MQTT_PACKETS_PINGRESP_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_PINGRESP, total);
        }
        MqttPacket::Disconnect(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_DISCONNECT_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_DISCONNECT_SENT, total);
        }
        MqttPacket::Auth(_, _) => {
            counter_metric_inc!(MQTT_PACKETS_AUTH_SENT, label_qos);
            counter_metric_inc!(MQTT_TOTAL_PACKETS_AUTH_SENT, total);
        }
        _ => unreachable!("This branch only matches for packets could not be sent"),
    }
}

pub fn record_retain_recv_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    let label = QosLabel { qos: qos_str };
    counter_metric_inc!(MQTT_RETAIN_PACKETS_RECEIVED, label)
}

pub fn record_retain_sent_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    let label = QosLabel { qos: qos_str };
    counter_metric_inc!(MQTT_RETAIN_PACKETS_SENT, label);
}

pub fn init() {
    // TotalLabel {} counters
    counter_metric_touch!(MQTT_TOTAL_BYTES_RECEIVED, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_BYTES_SENT, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_RECEIVED, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_SENT, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_CONNECT, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_CONNACK, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PUBLISH_RECEIVED, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PUBLISH_SENT, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PUBACK_RECEIVED, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PUBACK_SENT, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PUBREC_RECEIVED, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PUBREC_SENT, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PUBREL_RECEIVED, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PUBREL_SENT, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PUBCOMP_RECEIVED, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PUBCOMP_SENT, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_SUBSCRIBE, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_SUBACK, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_UNSUBSCRIBE, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_UNSUBACK, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PINGREQ, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_PINGRESP, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_DISCONNECT_RECEIVED, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_DISCONNECT_SENT, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_AUTH, TotalLabel {});
    counter_metric_touch!(MQTT_TOTAL_PACKETS_AUTH_SENT, TotalLabel {});

    // NetworkLabel counters — pre-register for each known network type
    for net in ALL_NETWORK_TYPES {
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_CONNECT_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_PUBLISH_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_CONNACK_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_PUBACK_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_PUBREC_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_PUBREL_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_PUBCOMP_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_SUBSCRIBE_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_UNSUBSCRIBE_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_PINGREQ_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_DISCONNECT_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_AUTH_RECEIVED, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_RECEIVED_ERROR, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_CONNACK_AUTH_ERROR, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_PACKETS_CONNACK_ERROR, label);
        let label = NetworkLabel {
            network: net.to_string(),
        };
        counter_metric_touch!(MQTT_BYTES_RECEIVED, label);

        // NetworkQosLabel — qos: "-1" (non-publish), "0", "1", "2"
        for qos in &["-1", "0", "1", "2"] {
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_CONNACK_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_PUBLISH_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_PUBACK_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_PUBREC_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_PUBREL_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_PUBCOMP_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_SUBACK_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_UNSUBACK_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_PINGRESP_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_DISCONNECT_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: "-1".to_string(),
            };
            counter_metric_touch!(MQTT_PACKETS_AUTH_SENT, label);
            let label = NetworkQosLabel {
                network: net.to_string(),
                qos: qos.to_string(),
            };
            counter_metric_touch!(MQTT_BYTES_SENT, label);
        }
    }

    // QosLabel — retain metrics for qos 0/1/2
    for qos in &["0", "1", "2"] {
        let label = QosLabel {
            qos: qos.to_string(),
        };
        counter_metric_touch!(MQTT_RETAIN_PACKETS_RECEIVED, label);
        let label = QosLabel {
            qos: qos.to_string(),
        };
        counter_metric_touch!(MQTT_RETAIN_PACKETS_SENT, label);
    }
}

#[cfg(test)]
mod test {
    use crate::core::server::metrics_register_default;

    use super::*;
    use common_base::tools::get_addr_by_local_hostname;
    use prometheus_client::encoding::text::encode;

    #[test]
    fn test_gauge() {
        let _guard = crate::METRIC_TEST_LOCK.lock().unwrap();
        let family = MQTT_PACKETS_RECEIVED.clone();
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let mut tasks = vec![];
            for tid in 0..100 {
                let family = family.clone();
                let task = tokio::spawn(async move {
                    let family = family.write().unwrap();
                    let gauge = family.get_or_create(&NetworkLabel {
                        network: format!("network-{tid}"),
                    });
                    gauge.inc();
                });
                tasks.push(task);
            }

            while let Some(task) = tasks.pop() {
                let _ = task.await;
            }
        });

        let family = family.read().unwrap();

        let gauge = family.get_or_create(&NetworkLabel {
            network: format!("network-{}", 0),
        });

        gauge.inc();
        assert_eq!(gauge.get(), 2);

        let mut buffer = String::new();

        let re = metrics_register_default();
        encode(&mut buffer, &re).unwrap();

        assert_ne!(buffer.capacity(), 0);
    }

    #[tokio::test]
    async fn test_gauge_metrics() {
        let _guard = crate::METRIC_TEST_LOCK.lock().unwrap();
        record_received_error_metrics(NetworkConnectionType::Tcp);
        let label = NetworkLabel {
            network: "tcp".to_string(),
        };
        {
            counter_metric_inc_by!(MQTT_PACKETS_RECEIVED_ERROR, label, 2);
        }

        {
            let c = MQTT_PACKETS_RECEIVED_ERROR
                .clone()
                .write()
                .unwrap()
                .get_or_create(&label)
                .get();
            assert_eq!(c, 2)
        }
    }

    use protocol::mqtt::codec::{calc_mqtt_packet_size, MqttPacketWrapper};
    use protocol::mqtt::common::{ConnAck, Connect, MqttPacket, Publish, UnsubAck};

    #[test]
    fn calc_mqtt_packet_size_test() {
        let _guard = crate::METRIC_TEST_LOCK.lock().unwrap();
        let unsub_ack = UnsubAck {
            pkid: 1,
            reasons: Vec::new(),
        };

        let packet = MqttPacket::UnsubAck(unsub_ack, None);
        let packet_wrapper = MqttPacketWrapper {
            protocol_version: 4,
            packet,
        };
        assert_eq!(calc_mqtt_packet_size(packet_wrapper), 4);
    }

    use bytes::Bytes;

    #[test]
    fn calc_mqtt_packet_test() {
        let _guard = crate::METRIC_TEST_LOCK.lock().unwrap();
        let mp: MqttPacket = MqttPacket::Publish(
            Publish {
                dup: false,
                qos: QoS::AtMostOnce,
                p_kid: 0,
                retain: false,
                topic: Bytes::from("test"),
                payload: Bytes::from("test"),
            },
            None,
        );

        let nc = NetworkConnection::new(
            NetworkConnectionType::Tcp,
            get_addr_by_local_hostname(1883).parse().unwrap(),
            None,
        );
        let ty = NetworkConnectionType::Tcp;
        let received_before = record_mqtt_total_packets_received_get();
        let publish_before = record_mqtt_total_packets_publish_received_get();
        record_packet_received_metrics(&nc, &mp, &ty);
        let label = NetworkLabel {
            network: ty.to_string(),
        };
        let pr = MQTT_PACKETS_RECEIVED.clone();
        {
            let rs1 = pr.read().unwrap().get_or_create(&label).get();
            assert_eq!(rs1, received_before + 1);
        }
        let ps = MQTT_PACKETS_PUBLISH_RECEIVED.clone();
        {
            let rs2 = ps.read().unwrap().get_or_create(&label).get();
            assert_eq!(rs2, publish_before + 1);
        }
    }

    #[test]
    fn records_receive_metrics_for_each_supported_packet_kind() {
        let _guard = crate::METRIC_TEST_LOCK.lock().unwrap();
        let connection = NetworkConnection::new(
            NetworkConnectionType::Tcp,
            get_addr_by_local_hostname(1883).parse().unwrap(),
            None,
        );
        let network = NetworkConnectionType::Tcp;
        let packets = [
            MqttPacket::ConnAck(
                ConnAck {
                    session_present: false,
                    code: ConnectReturnCode::Success,
                },
                None,
            ),
            MqttPacket::Publish(Publish::new("topic", "payload", false), None),
            MqttPacket::PubAck(
                protocol::mqtt::common::PubAck {
                    pkid: 1,
                    reason: None,
                },
                None,
            ),
            MqttPacket::PubRec(
                protocol::mqtt::common::PubRec {
                    pkid: 1,
                    reason: None,
                },
                None,
            ),
            MqttPacket::PubRel(
                protocol::mqtt::common::PubRel {
                    pkid: 1,
                    reason: None,
                },
                None,
            ),
            MqttPacket::PubComp(
                protocol::mqtt::common::PubComp {
                    pkid: 1,
                    reason: None,
                },
                None,
            ),
            MqttPacket::PingReq(protocol::mqtt::common::PingReq),
            MqttPacket::Disconnect(
                protocol::mqtt::common::Disconnect { reason_code: None },
                None,
            ),
            MqttPacket::Auth(protocol::mqtt::common::Auth { reason: None }, None),
            MqttPacket::Subscribe(
                protocol::mqtt::common::Subscribe {
                    packet_identifier: 1,
                    filters: Vec::new(),
                },
                None,
            ),
            MqttPacket::Unsubscribe(
                protocol::mqtt::common::Unsubscribe {
                    pkid: 1,
                    filters: Vec::new(),
                },
                None,
            ),
        ];

        for packet in &packets {
            record_packet_received_metrics(&connection, packet, &network);
        }

        assert!(record_mqtt_total_packets_received_get() >= packets.len() as u64);
        assert!(
            MQTT_PACKETS_CONNACK_RECEIVED
                .read()
                .unwrap()
                .get(&NetworkLabel {
                    network: network.to_string(),
                })
                .unwrap()
                .get()
                >= 1
        );
        assert!(record_mqtt_total_packets_publish_received_get() >= 1);
        assert!(record_mqtt_total_packets_puback_received_get() >= 1);
        assert!(record_mqtt_total_packets_pubrec_received_get() >= 1);
        assert!(record_mqtt_total_packets_pubrel_received_get() >= 1);
        assert!(record_mqtt_total_packets_pubcomp_received_get() >= 1);
        assert!(record_mqtt_total_packets_pingreq_get() >= 1);
        assert!(record_mqtt_total_packets_disconnect_received_get() >= 1);
        assert!(record_mqtt_total_packets_auth_get() >= 1);
        assert!(record_mqtt_total_packets_subscribe_get() >= 1);
        assert!(record_mqtt_total_packets_unsubscribe_get() >= 1);
    }

    #[test]
    fn records_send_metrics_for_ack_publish_and_error_packets() {
        let _guard = crate::METRIC_TEST_LOCK.lock().unwrap();
        let connack_before = record_mqtt_total_packets_connack_get();
        let auth_before = record_mqtt_total_packets_auth_sent_get();
        let wrappers = [
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::ConnAck(
                    ConnAck {
                        session_present: false,
                        code: ConnectReturnCode::NotAuthorized,
                    },
                    None,
                ),
            },
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::Publish(Publish::new("topic", "payload", false), None),
            },
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::PubAck(
                    protocol::mqtt::common::PubAck {
                        pkid: 1,
                        reason: Some(protocol::mqtt::common::PubAckReason::Success),
                    },
                    None,
                ),
            },
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::PubRec(
                    protocol::mqtt::common::PubRec {
                        pkid: 1,
                        reason: Some(protocol::mqtt::common::PubRecReason::Success),
                    },
                    None,
                ),
            },
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::PubRel(
                    protocol::mqtt::common::PubRel {
                        pkid: 1,
                        reason: Some(protocol::mqtt::common::PubRelReason::Success),
                    },
                    None,
                ),
            },
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::PubComp(
                    protocol::mqtt::common::PubComp {
                        pkid: 1,
                        reason: Some(protocol::mqtt::common::PubCompReason::Success),
                    },
                    None,
                ),
            },
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::SubAck(
                    protocol::mqtt::common::SubAck {
                        pkid: 1,
                        return_codes: Vec::new(),
                    },
                    None,
                ),
            },
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::UnsubAck(
                    protocol::mqtt::common::UnsubAck {
                        pkid: 1,
                        reasons: Vec::new(),
                    },
                    None,
                ),
            },
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::PingResp(protocol::mqtt::common::PingResp),
            },
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::Disconnect(
                    protocol::mqtt::common::Disconnect {
                        reason_code: Some(
                            protocol::mqtt::common::DisconnectReasonCode::NormalDisconnection,
                        ),
                    },
                    None,
                ),
            },
            MqttPacketWrapper {
                protocol_version: 5,
                packet: MqttPacket::Auth(
                    protocol::mqtt::common::Auth {
                        reason: Some(protocol::mqtt::common::AuthReason::Success),
                    },
                    None,
                ),
            },
        ];

        for wrapper in &wrappers {
            record_packet_send_metrics(wrapper, "coverage".to_string());
        }

        assert!(record_mqtt_total_packets_sent_get() >= wrappers.len() as u64);
        assert_eq!(record_mqtt_total_packets_connack_get(), connack_before + 1);
        assert_eq!(record_mqtt_total_packets_auth_sent_get(), auth_before + 1);
        assert_eq!(
            MQTT_PACKETS_AUTH_SENT
                .read()
                .unwrap()
                .get(&NetworkQosLabel {
                    network: "coverage".to_string(),
                    qos: "-1".to_string(),
                })
                .unwrap()
                .get(),
            1
        );
        assert!(record_mqtt_total_packets_publish_sent_get() >= 1);
        assert!(record_mqtt_total_packets_puback_sent_get() >= 1);
        assert!(record_mqtt_total_packets_pubrec_sent_get() >= 1);
        assert!(record_mqtt_total_packets_pubrel_sent_get() >= 1);
        assert!(record_mqtt_total_packets_pubcomp_sent_get() >= 1);
        assert!(record_mqtt_total_packets_suback_get() >= 1);
        assert!(record_mqtt_total_packets_unsuback_get() >= 1);
        assert!(record_mqtt_total_packets_pingresp_get() >= 1);
        assert!(record_mqtt_total_packets_disconnect_sent_get() >= 1);
    }

    #[test]
    fn init_preregisters_canonical_network_labels() {
        let _guard = crate::METRIC_TEST_LOCK.lock().unwrap();
        init();

        for network in ALL_NETWORK_TYPES {
            let label = NetworkLabel {
                network: network.to_string(),
            };
            assert!(MQTT_PACKETS_RECEIVED.read().unwrap().get(&label).is_some());
        }

        for legacy_label in ["WebSocket", "WebSockets", "QUIC"] {
            let label = NetworkLabel {
                network: legacy_label.to_string(),
            };
            assert!(MQTT_PACKETS_RECEIVED.read().unwrap().get(&label).is_none());
        }

        assert!(MQTT_PACKETS_AUTH_SENT
            .read()
            .unwrap()
            .get(&NetworkQosLabel {
                network: NetworkConnectionType::Tcp.to_string(),
                qos: "-1".to_string(),
            })
            .is_some());
    }

    #[test]
    fn records_connect_and_retain_metrics_and_reads_packet_totals() {
        let _guard = crate::METRIC_TEST_LOCK.lock().unwrap();
        let connection = NetworkConnection::new(
            NetworkConnectionType::Tcp,
            get_addr_by_local_hostname(1883).parse().unwrap(),
            None,
        );
        let mut connection = connection;
        connection.set_protocol(protocol::robust::RobustMQProtocol::MQTT5);
        let connect = MqttPacket::Connect(
            5,
            Connect {
                keep_alive: 60,
                client_id: "coverage-client".to_string(),
                clean_session: true,
            },
            None,
            None,
            None,
            None,
        );

        let received_bytes_before = record_mqtt_total_bytes_received_get();
        let packets_before = record_mqtt_total_packets_received_get();
        let connect_before = record_mqtt_total_packets_connect_get();
        record_packet_received_metrics(&connection, &connect, &NetworkConnectionType::Tcp);

        let packet_size = calc_mqtt_packet_size(MqttPacketWrapper {
            protocol_version: 5,
            packet: connect,
        }) as u64;
        assert!(record_mqtt_total_bytes_received_get() >= received_bytes_before + packet_size);
        assert_eq!(record_mqtt_total_packets_received_get(), packets_before + 1);
        assert_eq!(record_mqtt_total_packets_connect_get(), connect_before + 1);

        let sent_bytes_before = record_mqtt_total_bytes_sent_get();
        let sent_packet = MqttPacketWrapper {
            protocol_version: 5,
            packet: MqttPacket::ConnAck(
                ConnAck {
                    session_present: false,
                    code: ConnectReturnCode::Success,
                },
                None,
            ),
        };
        let sent_packet_size = calc_mqtt_packet_size(sent_packet.clone()) as u64;
        record_packet_send_metrics(&sent_packet, NetworkConnectionType::Tcp.to_string());
        assert!(record_mqtt_total_bytes_sent_get() >= sent_bytes_before + sent_packet_size);

        record_retain_recv_metrics(QoS::AtLeastOnce);
        record_retain_sent_metrics(QoS::ExactlyOnce);
        assert_eq!(
            MQTT_RETAIN_PACKETS_RECEIVED
                .read()
                .unwrap()
                .get(&QosLabel {
                    qos: "1".to_string()
                })
                .unwrap()
                .get(),
            1
        );
        assert_eq!(
            MQTT_RETAIN_PACKETS_SENT
                .read()
                .unwrap()
                .get(&QosLabel {
                    qos: "2".to_string()
                })
                .unwrap()
                .get(),
            1
        );
    }
}
