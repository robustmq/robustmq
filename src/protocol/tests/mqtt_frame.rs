#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use futures::{SinkExt, StreamExt};
    use protocol::{
        mqtt::{
            ConnAck, ConnAckProperties, Connect, ConnectProperties, ConnectReturnCode, LastWill,
            Login, MQTTPacket,
        },
        mqttv4::{self, codec::Mqtt4Codec},
        mqttv5::codec::Mqtt5Codec,
    };
    use tokio::net::{TcpListener, TcpStream};
    use tokio_util::codec::Framed;

    #[tokio::test]
    async fn mqtt4_frame_server() {
        let ip = "127.0.0.1:1228";
        let listener = TcpListener::bind(ip).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let mut stream = Framed::new(stream, Mqtt4Codec::new());
            tokio::spawn(async move {
                while let Some(Ok(data)) = stream.next().await {
                    println!("Got: {:?}", data);

                    // 发送的消息也只需要发送消息主体，不需要提供长度
                    // Framed/LengthDelimitedCodec 会自动计算并添加
                    //    let response = &data[0..5];
                    stream.send(build_mqtt4_pg_connect_ack()).await.unwrap();
                }
            });
        }
    }

    #[tokio::test]
    async fn mqtt4_frame_client() {
        let socket = TcpStream::connect("127.0.0.1:1228").await.unwrap();
        let mut stream: Framed<TcpStream, Mqtt4Codec> = Framed::new(socket, Mqtt4Codec::new());

        // send connect package
        let packet = build_mqtt4_pg_connect();
        let _ = stream.send(packet).await;

        let data = stream.next().await;

        println!("Got: {:?}", data.unwrap().unwrap());
    }

    /// Build the connect content package for the mqtt4 protocol
    fn build_mqtt4_pg_connect() -> MQTTPacket {
        let client_id = String::from("test_client_id");
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
        return MQTTPacket::Connect(connect, None, lastwill, None, login);
    }

    /// Build the connect content package for the mqtt4 protocol
    fn build_mqtt4_pg_connect_ack() -> MQTTPacket {
        let ack: ConnAck = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        return MQTTPacket::ConnAck(ack, None);
    }

    #[tokio::test]
    async fn mqtt5_frame_server() {
        let ip = "127.0.0.1:1228";
        let listener = TcpListener::bind(ip).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let mut stream = Framed::new(stream, Mqtt5Codec::new());
            tokio::spawn(async move {
                while let Some(Ok(data)) = stream.next().await {
                    println!("Got: {:?}", data);

                    // 发送的消息也只需要发送消息主体，不需要提供长度
                    // Framed/LengthDelimitedCodec 会自动计算并添加
                    //    let response = &data[0..5];
                    stream.send(build_mqtt5_pg_connect_ack()).await.unwrap();
                }
            });
        }
    }

    #[tokio::test]
    async fn mqtt5_frame_client() {
        let socket = TcpStream::connect("127.0.0.1:2228").await.unwrap();
        let mut stream: Framed<TcpStream, Mqtt5Codec> = Framed::new(socket, Mqtt5Codec::new());

        // send connect package
        let packet = build_mqtt5_pg_connect();
        let _ = stream.send(packet).await;

        let data = stream.next().await;

        println!("Got: {:?}", data.unwrap().unwrap());
    }

    /// Build the connect content package for the mqtt5 protocol
    fn build_mqtt5_pg_connect() -> MQTTPacket {
        let client_id = String::from("test_client_id");
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

        let mut properties = ConnectProperties::default();
        properties.session_expiry_interval = Some(30);
        return MQTTPacket::Connect(connect, Some(properties), lastwill, None, login);
    }

    /// Build the connect content package for the mqtt5 protocol
    fn build_mqtt5_pg_connect_ack() -> MQTTPacket {
        let ack: ConnAck = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        let mut properties = ConnAckProperties::default();
        properties.max_qos = Some(10u8);
        return MQTTPacket::ConnAck(ack, Some(properties));
    }
}
