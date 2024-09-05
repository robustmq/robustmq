use lazy_static::lazy_static;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use protocol::mqtt::{
    codec::{calc_mqtt_packet_size, MQTTPacketWrapper},
    common::{MQTTPacket, QoS},
};
use std::sync::Arc;

use crate::{
    handler::constant::{METRICS_KEY_NETWORK_TYPE, METRICS_KEY_QOS},
    server::{
        connection::{NetworkConnection, NetworkConnectionType},
        connection_manager::ConnectionManager,
        packet::ResponsePackage,
    },
};

lazy_static! {
    // Number of packets received
    static ref PACKETS_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "packets_received",
        "Number of packets received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of error packets received
    static ref PACKETS_RECEIVED_ERROR: IntGaugeVec = register_int_gauge_vec!(
        "packets_received_error",
        "Number of error packets received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets sent
    static ref PACKETS_SENT: IntGaugeVec = register_int_gauge_vec!(
        "packets_sent",
        "Number of packets sent",
        &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

    // Number of bytes received
    static ref BYTES_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "bytes_received",
        "Number of bytes received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of bytes sent
    static ref BYTES_SENT: IntGaugeVec = register_int_gauge_vec!(
        "bytes_sent",
        "Number of bytes sent",
        &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

    // Number of reserved messages received
    static ref RETAIN_PACKETS_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "retain_packets_received",
        "Number of reserved messages received",
        &[METRICS_KEY_QOS]
    )
    .unwrap();

    static ref RETAIN_PACKETS_SEND: IntGaugeVec = register_int_gauge_vec!(
        "retain_packets_sent",
        "Number of reserved messages sent",
        &[METRICS_KEY_QOS]
    )
    .unwrap();
}

// Record the packet-related metrics received by the server for failed resolution
pub fn record_received_error_metrics(network_type: NetworkConnectionType) {
    PACKETS_RECEIVED_ERROR
        .with_label_values(&[&network_type.to_string()])
        .inc();
}

// Record metrics related to packets received by the server
pub fn record_received_metrics(
    connection: &NetworkConnection,
    pkg: &MQTTPacket,
    network_type: &NetworkConnectionType,
) {
    let payload_size = if let Some(protocol) = connection.protocol.clone() {
        let wrapper = MQTTPacketWrapper {
            protocol_version: protocol.into(),
            packet: pkg.clone(),
        };
        calc_mqtt_packet_size(wrapper)
    } else {
        0
    };

    PACKETS_RECEIVED
        .with_label_values(&[&network_type.to_string()])
        .inc();

    BYTES_RECEIVED
        .with_label_values(&[&network_type.to_string()])
        .add(payload_size as i64);
}

// Record metrics related to messages pushed to the client
pub fn record_sent_metrics(resp: &ResponsePackage, connection_manager: &Arc<ConnectionManager>) {
    let qos_str = if let MQTTPacket::Publish(publish, _) = resp.packet.clone() {
        format!("{}", publish.qos as u8)
    } else {
        "-1".to_string()
    };

    let (payload_size, network_type) =
        if let Some(connection) = connection_manager.get_connect(resp.connection_id) {
            if let Some(protocol) = connection.protocol.clone() {
                let wrapper = MQTTPacketWrapper {
                    protocol_version: protocol.into(),
                    packet: resp.packet.clone(),
                };
                (
                    calc_mqtt_packet_size(wrapper),
                    connection.connection_type.to_string(),
                )
            } else {
                (0, "".to_string())
            }
        } else {
            (0, "".to_string())
        };

    PACKETS_SENT
        .with_label_values(&[&network_type, &qos_str])
        .inc();

    BYTES_SENT
        .with_label_values(&[&network_type, &qos_str])
        .add(payload_size as i64);
}

pub fn record_retain_recv_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    RETAIN_PACKETS_RECEIVED.with_label_values(&[&qos_str]).inc();
}

pub fn record_retain_sent_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    RETAIN_PACKETS_SEND.with_label_values(&[&qos_str]).inc();
}

#[cfg(test)]
mod tests {
    use protocol::mqtt::{
        codec::{calc_mqtt_packet_size, MQTTPacketWrapper},
        common::{MQTTPacket, UnsubAck},
    };

    #[test]
    fn calc_mqtt_packet_size_test() {
        let unsub_ack = UnsubAck {
            pkid: 1,
            reasons: Vec::new(),
        };

        let packet = MQTTPacket::UnsubAck(unsub_ack, None);
        let packet_wrapper = MQTTPacketWrapper {
            protocol_version: 4,
            packet,
        };
        assert_eq!(calc_mqtt_packet_size(packet_wrapper), 4);
    }
}
