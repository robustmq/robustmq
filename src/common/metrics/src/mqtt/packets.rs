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

use crate::{gauge_metric_inc, gauge_metric_inc_by, register_gauge_metric};

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

register_gauge_metric!(
    MQTT_PACKETS_RECEIVED,
    "mqtt_packets_received",
    "Number of MQTT packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_CONNECT_RECEIVED,
    "mqtt_packets_connect_received",
    "Number of MQTT CONNECT packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_PUBLISH_RECEIVED,
    "mqtt_packets_publish_received",
    "Number of MQTT PUBLISH packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_CONNACK_RECEIVED,
    "mqtt_packets_connack_received",
    "Number of MQTT CONNACK packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_PUBACK_RECEIVED,
    "mqtt_packets_puback_received",
    "Number of MQTT PUBACK packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_PUBREC_RECEIVED,
    "mqtt_packets_pubrec_received",
    "Number of MQTT PUBREC packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_PUBREL_RECEIVED,
    "mqtt_packets_pubrel_received",
    "Number of MQTT PUBREL packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_PUBCOMP_RECEIVED,
    "mqtt_packets_pubcomp_received",
    "Number of MQTT PUBCOMP packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_SUBSCRIBE_RECEIVED,
    "mqtt_packets_subscribe_received",
    "Number of MQTT SUBSCRIBE packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_UNSUBSCRIBE_RECEIVED,
    "mqtt_packets_unsubscribe_received",
    "Number of MQTT UNSUBSCRIBE packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_PINGREQ_RECEIVED,
    "mqtt_packets_pingreq_received",
    "Number of MQTT PINGREQ packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_DISCONNECT_RECEIVED,
    "mqtt_packets_disconnect_received",
    "Number of MQTT DISCONNECT packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_AUTH_RECEIVED,
    "mqtt_packets_auth_received",
    "Number of MQTT AUTH packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_RECEIVED_ERROR,
    "mqtt_packets_received_error",
    "Number of MQTT error packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_CONNACK_AUTH_ERROR,
    "mqtt_packets_connack_auth_error",
    "Number of MQTT CONNACK auth error packets received",
    NetworkLabel
);
register_gauge_metric!(
    MQTT_PACKETS_CONNACK_ERROR,
    "mqtt_packets_connack_error",
    "Number of MQTT CONNACK error packets received",
    NetworkLabel
);

register_gauge_metric!(
    MQTT_PACKETS_SENT,
    "mqtt_packets_sent",
    "Number of MQTT packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_PACKETS_CONNACK_SENT,
    "mqtt_packets_connack_sent",
    "Number of MQTT CONNACK packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_PACKETS_PUBLISH_SENT,
    "mqtt_packets_publish_sent",
    "Number of MQTT PUBLISH packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_PACKETS_PUBACK_SENT,
    "mqtt_packets_puback_sent",
    "Number of MQTT PUBACK packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_PACKETS_PUBREC_SENT,
    "mqtt_packets_pubrec_sent",
    "Number of MQTT PUBREC packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_PACKETS_PUBREL_SENT,
    "mqtt_packets_pubrel_sent",
    "Number of MQTT PUBREL packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_PACKETS_PUBCOMP_SENT,
    "mqtt_packets_pubcomp_sent",
    "Number of MQTT PUBCOMP packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_PACKETS_SUBACK_SENT,
    "mqtt_packets_suback_sent",
    "Number of MQTT SUBACK packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_PACKETS_UNSUBACK_SENT,
    "mqtt_packets_unsuback_sent",
    "Number of MQTT UNSUBACK packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_PACKETS_PINGRESP_SENT,
    "mqtt_packets_pingresp_sent",
    "Number of MQTT PINGRESP packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_PACKETS_DISCONNECT_SENT,
    "mqtt_packets_disconnect_sent",
    "Number of MQTT DISCONNECT packets sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_BYTES_RECEIVED,
    "mqtt_bytes_received",
    "Number of MQTT bytes received",
    NetworkLabel
);

register_gauge_metric!(
    MQTT_BYTES_SENT,
    "mqtt_bytes_sent",
    "Number of MQTT bytes sent",
    NetworkQosLabel
);

register_gauge_metric!(
    MQTT_RETAIN_PACKETS_RECEIVED,
    "mqtt_retain_packets_received",
    "Number of MQTT retain messages received",
    QosLabel
);

register_gauge_metric!(
    MQTT_RETAIN_PACKETS_SENT,
    "mqtt_retain_packets_sent",
    "Number of MQTT retain messages sent",
    QosLabel
);

register_gauge_metric!(
    MQTT_MESSAGES_DROPPED_NO_SUBSCRIBERS,
    "mqtt_messages_dropped_no_subscribers",
    "Number of MQTT messages dropped due to no subscribers",
    QosLabel
);

// Record the packet-related metrics received by the server for failed resolution
pub fn record_received_error_metrics(network_type: NetworkConnectionType) {
    let labe = NetworkLabel {
        network: network_type.to_string(),
    };
    gauge_metric_inc!(MQTT_PACKETS_RECEIVED_ERROR, labe);
}

// Record metrics related to packets received by the server
pub fn record_mqtt_packet_received_metrics(
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

    gauge_metric_inc_by!(MQTT_BYTES_RECEIVED, label, payload_size as i64);
    gauge_metric_inc!(MQTT_PACKETS_RECEIVED, label);

    match pkg {
        MqttPacket::Connect(_, _, _, _, _, _) => {
            gauge_metric_inc!(MQTT_PACKETS_CONNECT_RECEIVED, label)
        }
        MqttPacket::ConnAck(_, _) => {
            gauge_metric_inc!(MQTT_PACKETS_CONNACK_RECEIVED, label)
        }
        MqttPacket::Publish(_, _) => {
            gauge_metric_inc!(MQTT_PACKETS_PUBLISH_RECEIVED, label)
        }
        MqttPacket::PubAck(_, _) => gauge_metric_inc!(MQTT_PACKETS_PUBACK_RECEIVED, label),
        MqttPacket::PubRec(_, _) => gauge_metric_inc!(MQTT_PACKETS_PUBREC_RECEIVED, label),
        MqttPacket::PubRel(_, _) => gauge_metric_inc!(MQTT_PACKETS_PUBREL_RECEIVED, label),
        MqttPacket::PubComp(_, _) => {
            gauge_metric_inc!(MQTT_PACKETS_PUBCOMP_RECEIVED, label)
        }
        MqttPacket::PingReq(_) => gauge_metric_inc!(MQTT_PACKETS_PINGREQ_RECEIVED, label),
        MqttPacket::Disconnect(_, _) => {
            gauge_metric_inc!(MQTT_PACKETS_DISCONNECT_RECEIVED, label)
        }
        MqttPacket::Auth(_, _) => gauge_metric_inc!(MQTT_PACKETS_AUTH_RECEIVED, label),
        MqttPacket::Subscribe(_, _) => {
            gauge_metric_inc!(MQTT_PACKETS_SUBSCRIBE_RECEIVED, label)
        }
        MqttPacket::Unsubscribe(_, _) => {
            gauge_metric_inc!(MQTT_PACKETS_UNSUBSCRIBE_RECEIVED, label)
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
    gauge_metric_inc!(MQTT_PACKETS_SENT, label_qos);
    gauge_metric_inc_by!(MQTT_BYTES_SENT, label_qos, payload_size as i64);

    match &packet_wrapper.packet {
        MqttPacket::ConnAck(conn_ack, _) => {
            gauge_metric_inc!(MQTT_PACKETS_CONNACK_SENT, label_qos);
            //  NotAuthorized
            if conn_ack.code == ConnectReturnCode::NotAuthorized {
                gauge_metric_inc!(MQTT_PACKETS_CONNACK_AUTH_ERROR, label);
            }
            //  connack error
            if conn_ack.code != ConnectReturnCode::Success {
                gauge_metric_inc!(MQTT_PACKETS_CONNACK_ERROR, label);
            }
        }
        MqttPacket::Publish(_, _) => {
            gauge_metric_inc!(MQTT_PACKETS_PUBLISH_SENT, label_qos)
        }
        MqttPacket::PubAck(_, _) => gauge_metric_inc!(MQTT_PACKETS_PUBACK_SENT, label_qos),
        MqttPacket::PubRec(_, _) => gauge_metric_inc!(MQTT_PACKETS_PUBREC_SENT, label_qos),
        MqttPacket::PubRel(_, _) => gauge_metric_inc!(MQTT_PACKETS_PUBREL_SENT, label_qos),
        MqttPacket::PubComp(_, _) => {
            gauge_metric_inc!(MQTT_PACKETS_PUBCOMP_SENT, label_qos)
        }
        MqttPacket::SubAck(_, _) => gauge_metric_inc!(MQTT_PACKETS_SUBACK_SENT, label_qos),
        MqttPacket::UnsubAck(_, _) => {
            gauge_metric_inc!(MQTT_PACKETS_UNSUBACK_SENT, label_qos)
        }
        MqttPacket::PingResp(_) => gauge_metric_inc!(MQTT_PACKETS_PINGRESP_SENT, label_qos),
        MqttPacket::Disconnect(_, _) => {
            gauge_metric_inc!(MQTT_PACKETS_DISCONNECT_SENT, label_qos)
        }
        MqttPacket::Auth(_, _) => gauge_metric_inc!(MQTT_PACKETS_CONNACK_SENT, label_qos),
        _ => unreachable!("This branch only matches for packets could not be sent"),
    }
}

pub fn record_retain_recv_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    let label = QosLabel { qos: qos_str };
    gauge_metric_inc!(MQTT_RETAIN_PACKETS_RECEIVED, label)
}

pub fn record_retain_sent_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    let label = QosLabel { qos: qos_str };
    gauge_metric_inc!(MQTT_RETAIN_PACKETS_SENT, label);
}

pub fn record_messages_dropped_no_subscribers_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    let label = QosLabel { qos: qos_str };
    gauge_metric_inc!(MQTT_MESSAGES_DROPPED_NO_SUBSCRIBERS, label);
}

#[cfg(test)]
mod test {
    use crate::core::server::metrics_register_default;

    use super::*;
    use common_base::tools::{get_addr_by_local_hostname, now_second};
    use prometheus_client::encoding::text::encode;

    #[tokio::test]
    async fn test_gauge() {
        let family = MQTT_PACKETS_RECEIVED.clone();
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
            gauge_metric_inc_by!(MQTT_PACKETS_RECEIVED_ERROR, label, 2);
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
    use protocol::mqtt::common::{MqttPacket, Publish, UnsubAck};

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
    use protocol::robust::RobustMQProtocol;
    #[test]
    fn calc_mqtt_packet_test() {
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

        let nc = NetworkConnection {
            connection_type: NetworkConnectionType::Tcp,
            addr: get_addr_by_local_hostname(1883).parse().unwrap(),
            connection_stop_sx: None,
            connection_id: 100,
            protocol: Some(RobustMQProtocol::MQTT3),
            create_time: now_second(),
        };
        let ty = NetworkConnectionType::Tcp;
        record_mqtt_packet_received_metrics(&nc, &mp, &ty);
        let label = NetworkLabel {
            network: ty.to_string(),
        };
        let pr = MQTT_PACKETS_RECEIVED.clone();
        {
            let rs1 = pr.read().unwrap().get_or_create(&label).get();
            assert_eq!(rs1, 1);
        }
        let ps = MQTT_PACKETS_PUBLISH_RECEIVED.clone();
        {
            let rs2 = ps.read().unwrap().get_or_create(&label).get();
            assert_eq!(rs2, 1);
        }
    }
}
