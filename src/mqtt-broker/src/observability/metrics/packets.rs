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

use crate::server::connection::{NetworkConnection, NetworkConnectionType};
use prometheus_client::encoding::EncodeLabelSet;
use protocol::mqtt::{
    codec::{calc_mqtt_packet_size, MqttPacketWrapper},
    common::{ConnectReturnCode, MqttPacket, QoS},
};

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

common_base::register_gauge_metric!(
    PACKETS_RECEIVED,
    "packets_received",
    "Number of packets received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_CONNECT_RECEIVED,
    "packets_connect_received",
    "Number of packets connect received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_PUBLISH_RECEIVED,
    "packets_publish_received",
    "Number of packets publish received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_CONNACK_RECEIVED,
    "packets_connack_received",
    "Number of packets connack received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_PUBACK_RECEIVED,
    "packets_puback_received",
    "Number of packets puback received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_PUBREC_RECEIVED,
    "packets_pubrec_received",
    "Number of packets pubrec received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_PUBREL_RECEIVED,
    "packets_pubrel_received",
    "Number of packets pubrel received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_PUBCOMP_RECEIVED,
    "packets_pubcomp_received",
    "Number of packets pubcomp received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_SUBSCRIBE_RECEIVED,
    "packets_subscribe_received",
    "Number of packets subscribe received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_UNSUBSCRIBE_RECEIVED,
    "packets_unsubscribe_received",
    "Number of packets unsubscribe received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_PINGREQ_RECEIVED,
    "packets_pingreq_received",
    "Number of packets pingreq received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_DISCONNECT_RECEIVED,
    "packets_disconnect_received",
    "Number of packets disconnect received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_AUTH_RECEIVED,
    "packets_auth_received",
    "Number of packets auth received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_RECEIVED_ERROR,
    "packets_received_error",
    "Number of error packets received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_CONNACK_AUTH_ERROR,
    "packets_connack_auth_error",
    "Number of connack auth error packets received",
    NetworkLabel
);
common_base::register_gauge_metric!(
    PACKETS_CONNACK_ERROR,
    "packets_connack_error",
    "Number of connack error packets received",
    NetworkLabel
);

common_base::register_gauge_metric!(
    PACKETS_SENT,
    "packets_sent",
    "Number of packets sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_CONNACK_SENT,
    "packets_connack_sent",
    "Number of packets connack sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_PUBLISH_SENT,
    "packets_publish_sent",
    "Number of packets publish sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_PUBACK_SENT,
    "packets_puback_sent",
    "Number of packets puback sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_PUBREC_SENT,
    "packets_pubrec_sent",
    "Number of packets pubrec sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_PUBREL_SENT,
    "packets_pubrel_sent",
    "Number of packets pubrel sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_PUBCOMP_SENT,
    "packets_pubcomp_sent",
    "Number of packets pubcomp sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_SUBACK_SENT,
    "packets_suback_sent",
    "Number of packets suback sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_UNSUBACK_SENT,
    "packets_unsuback_sent",
    "Number of packets unsuback sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_PINGRESP_SENT,
    "packets_pingresp_sent",
    "Number of packets pingresp sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_DISCONNECT_SENT,
    "packets_disconnect_sent",
    "Number of packets disconnect sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    PACKETS_AUTH_SENT,
    "packets_auth_sent",
    "Number of packets auth sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    BYTES_RECEIVED,
    "bytes_received",
    "Number of bytes received",
    NetworkLabel
);

common_base::register_gauge_metric!(
    BYTES_SENT,
    "bytes_sent",
    "Number of bytes sent",
    NetworkQosLabel
);

common_base::register_gauge_metric!(
    RETAIN_PACKETS_RECEIVED,
    "retain_packets_received",
    "Number of reserved messages received",
    QosLabel
);

common_base::register_gauge_metric!(
    RETAIN_PACKETS_SEND,
    "retain_packets_sent",
    "Number of reserved messages sent",
    QosLabel
);

common_base::register_gauge_metric!(
    MESSAGES_DROPPED_NO_SUBSCRIBERS,
    "messages_dropped_no_subscribers",
    "Number of messages dropped due to no subscribers",
    QosLabel
);

common_base::register_gauge_metric!(
    MESSAGES_DROPPED_DISCARD,
    "messages_dropped_discard",
    "Number of messages dropped due to discard",
    QosLabel
);

// Record the packet-related metrics received by the server for failed resolution
pub fn record_received_error_metrics(network_type: NetworkConnectionType) {
    let labe = NetworkLabel {
        network: network_type.to_string(),
    };
    common_base::gauge_metric_inc!(PACKETS_RECEIVED_ERROR, labe);
}

// Record metrics related to packets received by the server
pub fn record_received_metrics(
    connection: &NetworkConnection,
    pkg: &MqttPacket,
    network_type: &NetworkConnectionType,
) {
    let label = NetworkLabel {
        network: network_type.to_string(),
    };
    let payload_size = if let Some(protocol) = connection.protocol.clone() {
        let wrapper = MqttPacketWrapper {
            protocol_version: protocol.into(),
            packet: pkg.clone(),
        };
        calc_mqtt_packet_size(wrapper)
    } else {
        0
    };
    common_base::gauge_metric_inc_by!(BYTES_RECEIVED, label, payload_size as i64);
    common_base::gauge_metric_inc!(PACKETS_RECEIVED, label);
    match pkg {
        MqttPacket::Connect(_, _, _, _, _, _) => {
            common_base::gauge_metric_inc!(PACKETS_CONNECT_RECEIVED, label)
        }
        MqttPacket::ConnAck(_, _) => {
            common_base::gauge_metric_inc!(PACKETS_CONNACK_RECEIVED, label)
        }
        MqttPacket::Publish(_, _) => {
            common_base::gauge_metric_inc!(PACKETS_PUBLISH_RECEIVED, label)
        }
        MqttPacket::PubAck(_, _) => common_base::gauge_metric_inc!(PACKETS_PUBACK_RECEIVED, label),
        MqttPacket::PubRec(_, _) => common_base::gauge_metric_inc!(PACKETS_PUBREC_RECEIVED, label),
        MqttPacket::PubRel(_, _) => common_base::gauge_metric_inc!(PACKETS_PUBREL_RECEIVED, label),
        MqttPacket::PubComp(_, _) => {
            common_base::gauge_metric_inc!(PACKETS_PUBCOMP_RECEIVED, label)
        }
        MqttPacket::PingReq(_) => common_base::gauge_metric_inc!(PACKETS_PINGREQ_RECEIVED, label),
        MqttPacket::Disconnect(_, _) => {
            common_base::gauge_metric_inc!(PACKETS_DISCONNECT_RECEIVED, label)
        }
        MqttPacket::Auth(_, _) => common_base::gauge_metric_inc!(PACKETS_AUTH_RECEIVED, label),
        MqttPacket::Subscribe(_, _) => {
            common_base::gauge_metric_inc!(PACKETS_SUBSCRIBE_RECEIVED, label)
        }
        MqttPacket::Unsubscribe(_, _) => {
            common_base::gauge_metric_inc!(PACKETS_UNSUBSCRIBE_RECEIVED, label)
        }
        _ => unreachable!("This branch only matches for packets could not be received"),
    }
}

// Record metrics related to messages pushed to the client
pub fn record_sent_metrics(packet_wrapper: &MqttPacketWrapper, network_type: String) {
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
        network: network_type.clone(),
    };
    let payload_size = calc_mqtt_packet_size(packet_wrapper.to_owned());
    common_base::gauge_metric_inc!(PACKETS_SENT, label_qos);
    common_base::gauge_metric_inc_by!(BYTES_SENT, label_qos, payload_size as i64);

    match &packet_wrapper.packet {
        MqttPacket::ConnAck(conn_ack, _) => {
            common_base::gauge_metric_inc!(PACKETS_CONNACK_SENT, label_qos);
            //  NotAuthorized
            if conn_ack.code == ConnectReturnCode::NotAuthorized {
                common_base::gauge_metric_inc!(PACKETS_CONNACK_AUTH_ERROR, label);
            }
            //  connack error
            if conn_ack.code != ConnectReturnCode::Success {
                common_base::gauge_metric_inc!(PACKETS_CONNACK_ERROR, label);
            }
        }
        MqttPacket::Publish(_, _) => {
            common_base::gauge_metric_inc!(PACKETS_PUBLISH_SENT, label_qos)
        }
        MqttPacket::PubAck(_, _) => common_base::gauge_metric_inc!(PACKETS_PUBACK_SENT, label_qos),
        MqttPacket::PubRec(_, _) => common_base::gauge_metric_inc!(PACKETS_PUBREC_SENT, label_qos),
        MqttPacket::PubRel(_, _) => common_base::gauge_metric_inc!(PACKETS_PUBREL_SENT, label_qos),
        MqttPacket::PubComp(_, _) => {
            common_base::gauge_metric_inc!(PACKETS_PUBCOMP_SENT, label_qos)
        }
        MqttPacket::SubAck(_, _) => common_base::gauge_metric_inc!(PACKETS_SUBACK_SENT, label_qos),
        MqttPacket::UnsubAck(_, _) => {
            common_base::gauge_metric_inc!(PACKETS_UNSUBACK_SENT, label_qos)
        }
        MqttPacket::PingResp(_) => common_base::gauge_metric_inc!(PACKETS_PINGRESP_SENT, label_qos),
        MqttPacket::Disconnect(_, _) => {
            common_base::gauge_metric_inc!(PACKETS_DISCONNECT_SENT, label_qos)
        }
        MqttPacket::Auth(_, _) => common_base::gauge_metric_inc!(PACKETS_CONNACK_SENT, label_qos),
        _ => unreachable!("This branch only matches for packets could not be sent"),
    }
}

pub fn record_retain_recv_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    let label = QosLabel { qos: qos_str };
    common_base::gauge_metric_inc!(RETAIN_PACKETS_RECEIVED, label)
}

pub fn record_retain_sent_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    let label = QosLabel { qos: qos_str };
    common_base::gauge_metric_inc!(RETAIN_PACKETS_SEND, label);
}

pub fn record_messages_dropped_no_subscribers_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    let label = QosLabel { qos: qos_str };
    common_base::gauge_metric_inc!(MESSAGES_DROPPED_NO_SUBSCRIBERS, label);
}

pub fn record_messages_dropped_discard_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    let label = QosLabel { qos: qos_str };
    common_base::gauge_metric_inc!(MESSAGES_DROPPED_DISCARD, label);
}

#[cfg(test)]
mod test {
    use super::*;
    use common_base::metrics::metrics_register_default;
    use common_base::tools::get_addr_by_local_hostname;
    use prometheus_client::encoding::text::encode;
    #[tokio::test]
    async fn test_gauge() {
        let family = PACKETS_RECEIVED.clone();
        let mut tasks = vec![];
        for tid in 0..100 {
            let family = family.clone();
            let task = tokio::spawn(async move {
                let family = family.write().unwrap();
                let gauge = family.get_or_create(&NetworkLabel {
                    network: format!("network-{}", tid),
                });
                gauge.inc();
            });
            tasks.push(task);
        }

        // 逐个 `await` 任务
        while let Some(task) = tasks.pop() {
            let _ = task.await;
        }

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
        record_received_error_metrics(NetworkConnectionType::Tcp);
        let label = NetworkLabel {
            network: "tcp".to_string(),
        };
        {
            common_base::gauge_metric_inc_by!(PACKETS_RECEIVED_ERROR, label, 2);
        }

        {
            let c = PACKETS_RECEIVED_ERROR
                .clone()
                .write()
                .unwrap()
                .get_or_create(&label)
                .get();
            assert_eq!(c, 3)
        }
    }

    use protocol::mqtt::codec::{calc_mqtt_packet_size, MqttPacketWrapper};
    use protocol::mqtt::common::{MqttPacket, MqttProtocol, Publish, UnsubAck};

    #[test]
    fn calc_mqtt_packet_size_test() {
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
        let mp: MqttPacket = MqttPacket::Publish(
            Publish {
                dup: false,
                qos: QoS::AtMostOnce,
                pkid: 0,
                retain: false,
                topic: Bytes::from("test"),
                payload: Bytes::from("test"),
            },
            None,
        );

        let nc = NetworkConnection {
            connection_type: NetworkConnectionType::Tcp,
            addr: get_addr_by_local_hostname(1883).parse().unwrap(),
            connection_stop_sx: None,
            connection_id: 100,
            protocol: Some(MqttProtocol::Mqtt3),
        };
        let ty = NetworkConnectionType::Tcp;
        record_received_metrics(&nc, &mp, &ty);
        let label = NetworkLabel {
            network: ty.to_string(),
        };
        let pr = PACKETS_RECEIVED.clone();
        {
            let rs1 = pr.read().unwrap().get_or_create(&label).get();
            assert_eq!(rs1, 1);
        }
        let ps = PACKETS_PUBLISH_RECEIVED.clone();
        {
            let rs2 = ps.read().unwrap().get_or_create(&label).get();
            assert_eq!(rs2, 1);
        }
    }
}
