use bytes::BytesMut;
use protocol::{mqtt::{ConnAck, ConnectReturnCode}, mqttv4::{ MqttV4, connack}};

// build connack package
pub fn package_ack_write(sp:bool, code: ConnectReturnCode)  -> BytesMut {
    let connack: ConnAck = ConnAck {
        session_present: sp,
        code: code,
    };
    let mut buffer = BytesMut::new();
    
    // test the write function
    let _ = connack::write(&connack, &mut buffer);
    return buffer;
}
