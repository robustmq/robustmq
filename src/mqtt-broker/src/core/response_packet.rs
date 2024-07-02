use metadata_struct::mqtt::cluster::MQTTCluster;
use protocol::mqtt::common::{
    ConnAck, ConnAckProperties, ConnectProperties, ConnectReturnCode, MQTTPacket,
};

use super::{connection::response_information, validator::is_request_problem_info};

pub fn response_packet_matt5_connect_success(
    cluster: &MQTTCluster,
    client_id: String,
    auto_client_id: bool,
    session_expiry_interval: u32,
    session_present: bool,
    connect_properties: &Option<ConnectProperties>,
) -> MQTTPacket {
    let assigned_client_identifier = if auto_client_id {
        Some(client_id)
    } else {
        None
    };

    let properties = ConnAckProperties {
        session_expiry_interval: Some(session_expiry_interval),
        receive_max: Some(cluster.receive_max()),
        max_qos: Some(cluster.max_qos().into()),
        retain_available: Some(cluster.retain_available()),
        max_packet_size: Some(cluster.max_packet_size()),
        assigned_client_identifier: assigned_client_identifier,
        topic_alias_max: Some(cluster.topic_alias_max()),
        reason_string: None,
        user_properties: Vec::new(),
        wildcard_subscription_available: Some(cluster.wildcard_subscription_available()),
        subscription_identifiers_available: Some(cluster.subscription_identifiers_available()),
        shared_subscription_available: Some(cluster.shared_subscription_available()),
        server_keep_alive: Some(cluster.server_keep_alive()),
        response_information: response_information(connect_properties),
        server_reference: None,
        authentication_method: None,
        authentication_data: None,
    };
    return MQTTPacket::ConnAck(
        ConnAck {
            session_present,
            code: ConnectReturnCode::Success,
        },
        Some(properties),
    );
}

pub fn response_packet_matt5_connect_fail(
    code: ConnectReturnCode,
    connect_properties: &Option<ConnectProperties>,
    error: Option<String>,
) -> MQTTPacket {
    let mut ack_properties = ConnAckProperties::default();
    if is_request_problem_info(connect_properties) {
        ack_properties.reason_string = error;
    }
    return MQTTPacket::ConnAck(
        ConnAck {
            session_present: false,
            code,
        },
        Some(ack_properties),
    );
}
