use protocol::mqtt::{ConnAck, ConnectReturnCode, MQTTPacket};

pub fn conn_ack() -> MQTTPacket {
    let conn_ack = ConnAck {
        session_present: true,
        code: ConnectReturnCode::Success,
    };
    return MQTTPacket::ConnAck(conn_ack, None);
}
