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
    use crate::mqtt::protocol::common::{build_client_id, password};
    use bytes::Bytes;
    use common_base::tools::now_second;
    use futures::{SinkExt, StreamExt};
    use protocol::mqtt::common::{Connect, ConnectProperties, LastWill, Login, MqttPacket};
    use protocol::mqtt::mqttv4::codec::Mqtt4Codec;
    use protocol::mqtt::mqttv5::codec::Mqtt5Codec;
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio::time::{sleep, timeout};
    use tokio_util::codec::Framed;
    use tokio_util::time::FutureExt;

    #[tokio::test]
    async fn mqtt34_properties_test() {
        let socket = TcpStream::connect("127.0.0.1:1883")
            .timeout(Duration::from_secs(3))
            .await
            .unwrap()
            .unwrap();
        let mut stream: Framed<TcpStream, Mqtt4Codec> = Framed::new(socket, Mqtt4Codec::new());

        // send connect package
        let packet = build_mqtt_pg_connect(false);
        stream.send(packet).await.unwrap();
        let wait_res = timeout(Duration::from_secs(15), async {
            loop {
                let Some(data) = stream.next().await else {
                    sleep(Duration::from_secs(1)).await;
                    continue;
                };

                match data {
                    Ok(pkt) => match pkt {
                        MqttPacket::ConnAck(_, properties) => {
                            assert!(properties.is_none());
                            return Ok(());
                        }
                        _ => {
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    },

                    Err(e) => {
                        // When the peer closes the connection, the decoder might see an incomplete frame.
                        // Treat that as non-fatal and keep waiting for a proper DISCONNECT.
                        if !e.to_string().contains("Insufficient number") {
                            return Err(format!(
                                "decode error before DISCONNECT (ts={}): {}",
                                now_second(),
                                e
                            ));
                        }
                    }
                }
            }
        })
        .await;

        match wait_res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => panic!("{e}"),
            Err(_) => panic!("timeout waiting for DISCONNECT from broker"),
        }
    }

    #[tokio::test]
    async fn mqtt5_properties_test() {
        let socket = TcpStream::connect("127.0.0.1:1883")
            .timeout(Duration::from_secs(3))
            .await
            .unwrap()
            .unwrap();
        let mut stream: Framed<TcpStream, Mqtt5Codec> = Framed::new(socket, Mqtt5Codec::new());

        // send connect package
        let packet = build_mqtt_pg_connect(true);
        stream.send(packet).await.unwrap();
        let wait_res = timeout(Duration::from_secs(15), async {
            loop {
                let Some(data) = stream.next().await else {
                    sleep(Duration::from_secs(1)).await;
                    continue;
                };

                match data {
                    Ok(pkt) => match pkt {
                        MqttPacket::ConnAck(_, properties) => {
                            assert!(properties.is_some());
                            return Ok(());
                        }
                        _ => {
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    },

                    Err(e) => {
                        // When the peer closes the connection, the decoder might see an incomplete frame.
                        // Treat that as non-fatal and keep waiting for a proper DISCONNECT.
                        if !e.to_string().contains("Insufficient number") {
                            return Err(format!(
                                "decode error before DISCONNECT (ts={}): {}",
                                now_second(),
                                e
                            ));
                        }
                    }
                }
            }
        })
        .await;

        match wait_res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => panic!("{e}"),
            Err(_) => panic!("timeout waiting for DISCONNECT from broker"),
        }
    }

    fn build_mqtt_pg_connect(mqtt5: bool) -> MqttPacket {
        let client_id = build_client_id("mqtt5_keep_alive_test");
        let login = Some(Login {
            username: "admin".to_string(),
            password: password(),
        });
        let lastwill = Some(LastWill {
            topic: Bytes::from("topic1"),
            message: Bytes::from("connection content"),
            qos: protocol::mqtt::common::QoS::AtLeastOnce,
            retain: true,
        });

        let connect: Connect = Connect {
            keep_alive: 30,
            client_id,
            clean_session: true,
        };

        let properties = ConnectProperties {
            ..Default::default()
        };

        if mqtt5 {
            MqttPacket::Connect(5, connect, Some(properties), lastwill, None, login)
        } else {
            MqttPacket::Connect(5, connect, None, lastwill, None, login)
        }
    }
}
