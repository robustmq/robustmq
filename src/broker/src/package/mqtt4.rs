use bytes::BytesMut;
use protocol::{protocol::{ConnAck, ConnectReturnCode}, mqttv4::{ MqttV4, connack}};

// build connack package
pub fn package_ack_write() -> BytesMut {
    let connack: ConnAck = ConnAck {
        session_present: false,
        code: ConnectReturnCode::Success,
    };
    let mut buffer = BytesMut::new();
    
    // test the write function
    let _ = connack::write(&connack, &mut buffer);
    return buffer;
}
