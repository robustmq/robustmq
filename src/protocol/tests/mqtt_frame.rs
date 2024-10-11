// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use futures::{SinkExt, StreamExt};
    use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
    use protocol::mqtt::common::{
        ConnAck, ConnAckProperties, Connect, ConnectProperties, ConnectReturnCode, LastWill, Login,
        MQTTPacket,
    };
    use protocol::mqtt::mqttv4::codec::Mqtt4Codec;
    use protocol::mqtt::mqttv5::codec::Mqtt5Codec;
    use tokio::io;
    use tokio::net::{TcpListener, TcpStream};
    use tokio_util::codec::{Framed, FramedRead, FramedWrite};

    #[tokio::test]
    #[ignore]
    async fn mqtt_frame_server() {
        let ip = "127.0.0.1:1884";
        let listener = TcpListener::bind(ip).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let (r_stream, w_stream) = io::split(stream);
            let codec = MqttCodec::new(None);
            let mut read_frame_stream = FramedRead::new(r_stream, codec.clone());
            let mut write_frame_stream = FramedWrite::new(w_stream, codec.clone());
            tokio::spawn(async move {
                while let Some(Ok(data)) = read_frame_stream.next().await {
                    println!("Got: {:?}", data);

                    // 发送的消息也只需要发送消息主体，不需要提供长度
                    // Framed/LengthDelimitedCodec 会自动计算并添加
                    //    let response = &data[0..5];
                    write_frame_stream
                        .send(build_mqtt5_pg_connect_ack())
                        .await
                        .unwrap();
                }
            });
        }
    }

    #[tokio::test]
    #[ignore]
    async fn mqtt4_frame_server() {
        let ip = "127.0.0.1:1884";
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
                    //stream.send(build_mqtt4_pg_connect_ack()).await.unwrap();
                }
            });
        }
    }

    #[tokio::test]
    async fn mqtt4_frame_client() {
        let socket = TcpStream::connect("127.0.0.1:1883").await.unwrap();
        let mut stream: Framed<TcpStream, Mqtt4Codec> = Framed::new(socket, Mqtt4Codec::new());

        // send connect package
        let packet = build_mqtt4_pg_connect();
        let _ = stream.send(packet).await;

        if let Some(data) = stream.next().await {
            match data {
                Ok(da) => {
                    println!("success:{:?}", da);
                }
                Err(e) => {
                    println!("error:{}", e);
                }
            }
        }
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
            qos: protocol::mqtt::common::QoS::AtLeastOnce,
            retain: true,
        });

        let connect: Connect = Connect {
            keep_alive: 30u16, // 30 seconds
            client_id,
            clean_session: true,
        };
        MQTTPacket::Connect(4, connect, None, lastwill, None, login)
    }

    /// Build the connect content package for the mqtt4 protocol
    #[allow(dead_code)]
    fn build_mqtt4_pg_connect_ack() -> MqttPacketWrapper {
        let ack: ConnAck = ConnAck {
            session_present: false,
            code: ConnectReturnCode::Success,
        };
        MqttPacketWrapper {
            protocol_version: 4,
            packet: MQTTPacket::ConnAck(ack, None),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn mqtt5_frame_server() {
        let ip = "127.0.0.1:1884";
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
                    // stream.send(build_mqtt5_pg_connect_ack()).await.unwrap();
                }
            });
        }
    }

    #[tokio::test]
    async fn mqtt5_frame_client() {
        let socket = TcpStream::connect("127.0.0.1:1883").await.unwrap();
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
            qos: protocol::mqtt::common::QoS::AtLeastOnce,
            retain: true,
        });

        let connect: Connect = Connect {
            keep_alive: 30u16, // 30 seconds
            client_id,
            clean_session: true,
        };

        let properties = ConnectProperties {
            session_expiry_interval: Some(30),
            ..Default::default()
        };
        MQTTPacket::Connect(5, connect, Some(properties), lastwill, None, login)
    }

    /// Build the connect content package for the mqtt5 protocol
    fn build_mqtt5_pg_connect_ack() -> MqttPacketWrapper {
        let ack: ConnAck = ConnAck {
            session_present: true,
            code: ConnectReturnCode::Success,
        };
        let properties = ConnAckProperties {
            max_qos: Some(10u8),
            ..Default::default()
        };
        MqttPacketWrapper {
            protocol_version: 5,
            packet: MQTTPacket::ConnAck(ack, Some(properties)),
        }
    }
}
