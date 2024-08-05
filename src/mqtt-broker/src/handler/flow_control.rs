use protocol::mqtt::common::{MQTTProtocol, QoS};

pub fn is_flow_control(protocol: &MQTTProtocol, qos: QoS) -> bool {
    return protocol.is_mqtt5() && (qos == QoS::AtLeastOnce || qos == QoS::ExactlyOnce);
}

pub fn is_connection_rate_exceeded() -> bool {
    return false;
}

pub fn is_subscribe_rate_exceeded() -> bool {
    return false;
}
