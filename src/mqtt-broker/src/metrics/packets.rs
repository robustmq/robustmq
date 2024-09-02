use lazy_static::lazy_static;
use prometheus::{register_int_gauge_vec, IntGaugeVec};

use crate::{
    handler::constant::METRICS_KEY_NETWORK_TYPE, server::connection::NetworkConnectionType,
};

lazy_static! {
    // Number of packets received
    static ref PACKETS_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "packets.received",
        "Number of packets received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of error packets received
    static ref PACKETS_RECEIVED_ERROR: IntGaugeVec = register_int_gauge_vec!(
        "packets.received.error",
        "Number of error packets received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets sent
    static ref PACKETS_SENT: IntGaugeVec = register_int_gauge_vec!(
        "packets.sent",
        "Number of packets sent",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of bytes received
    static ref BYTES_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "bytes.received",
        "Number of bytes received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of bytes sent
    static ref BYTES_SENT: IntGaugeVec = register_int_gauge_vec!(
        "bytes.sent",
        "Number of bytes sent",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();
}

pub fn metrics_request_error_packet_incr(network_type: NetworkConnectionType) {
    PACKETS_RECEIVED_ERROR
        .with_label_values(&[&network_type.to_string()])
        .inc();
}

pub fn metrics_request_packet_incr(network_type: NetworkConnectionType) {
    PACKETS_RECEIVED
        .with_label_values(&[&network_type.to_string()])
        .inc();
}
