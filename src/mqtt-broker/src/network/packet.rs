use protocol::mqtt::{ConnAck, ConnectReturnCode, Packet};

pub fn conn_ack() -> Packet {
    let conn_ack = ConnAck {
        session_present: true,
        code: ConnectReturnCode::Success,
    };
    return Packet::ConnAck(conn_ack, None);
}
