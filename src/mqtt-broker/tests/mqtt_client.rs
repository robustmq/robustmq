#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};

    use protocol::{
        mqtt::{Connect, LastWill, Login},
        mqttv4,
    };
    #[test]
    fn mqtt4_broker() {}

    /// Build the connect content package for the mqtt4 protocol
    fn build_pg_connect() -> BytesMut {
        let client_id = String::from("test_client_id");
        let mut buff_write: BytesMut = BytesMut::new();
        let login = Some(Login {
            username: "lobo".to_string(),
            password: "123456".to_string(),
        });
        let lastwill = Some(LastWill {
            topic: Bytes::from("topic1"),
            message: Bytes::from("connection content"),
            qos: protocol::mqtt::QoS::AtLeastOnce,
            retain: true,
        });

        let connect: Connect = Connect {
            keep_alive: 30u16, // 30 seconds
            client_id: client_id,
            clean_session: true,
        };
        let _ = mqttv4::connect::write(&connect, &login, &lastwill, &mut buff_write);
        return buff_write;
    }
}
