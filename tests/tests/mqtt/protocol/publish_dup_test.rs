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
    use common_base::uuid::unique_id;
    use futures::{SinkExt, StreamExt};
    use protocol::mqtt::common::{
        Connect, Login, MqttPacket, PubCompReason, PubRel, PubRelReason, Publish, QoS,
    };
    use protocol::mqtt::mqttv5::codec::Mqtt5Codec;
    use std::time::{Duration, Instant};
    use tokio::net::TcpStream;
    use tokio_util::codec::Framed;
    use tokio_util::time::FutureExt;

    #[tokio::test]
    async fn publish_qos1_dup_test() {
        let socket = TcpStream::connect("127.0.0.1:1883")
            .timeout(Duration::from_secs(3))
            .await
            .unwrap()
            .unwrap();
        let mut stream: Framed<TcpStream, Mqtt5Codec> = Framed::new(socket, Mqtt5Codec::new());
        let topic = format!("/publish_qos1_dup_test/{}", unique_id());
        let message = "publish_qos1_dup_test mqtt message".to_string();
        // send connect package
        let packet = build_mqtt5_pg_connect();
        stream.send(packet).await.unwrap();
        stream.next().await;

        // publish data
        let pub_packet = MqttPacket::Publish(
            Publish {
                dup: false,
                qos: QoS::AtLeastOnce,
                p_kid: 1,
                retain: false,
                topic: Bytes::from(topic.to_string()),
                payload: Bytes::from(message.to_string()),
            },
            None,
        );

        for _ in 0..10 {
            stream.send(pub_packet.clone()).await.unwrap();
        }

        let mut resp_data = Vec::new();
        for _ in 0..10 {
            if let Some(resp) = stream.next().await {
                if resp.is_err() {
                    continue;
                }
                let resp_packet = resp.unwrap();
                if let MqttPacket::PubAck(_, ack_pros) = resp_packet {
                    let user_properties = ack_pros.unwrap().user_properties;
                    if !user_properties.is_empty() {
                        let dump_flag = user_properties.first().unwrap();
                        if "PUBLISH_QOS_DUMP" == dump_flag.0 && "true" == dump_flag.1 {
                            resp_data.push(true);
                        }
                    }
                }
            }
        }

        assert!(!resp_data.is_empty());
    }

    #[tokio::test]
    async fn publish_qos2_publish_dup_test() {
        let socket = TcpStream::connect("127.0.0.1:1883")
            .timeout(Duration::from_secs(3))
            .await
            .unwrap()
            .unwrap();
        let mut stream: Framed<TcpStream, Mqtt5Codec> = Framed::new(socket, Mqtt5Codec::new());
        let topic = format!("/publish_qos2_publish_dup_test/{}", unique_id(),);
        let message = "publish_qos2_publish_dup_test mqtt message".to_string();

        // send connect package
        let packet = build_mqtt5_pg_connect();
        stream.send(packet).await.unwrap();
        stream.next().await;

        // publish data
        let pub_packet = MqttPacket::Publish(
            Publish {
                dup: false,
                qos: QoS::ExactlyOnce,
                p_kid: 1,
                retain: false,
                topic: Bytes::from(topic.to_string()),
                payload: Bytes::from(message.to_string()),
            },
            None,
        );

        for _ in 0..10 {
            stream.send(pub_packet.clone()).await.unwrap();
        }

        let mut resp_data = Vec::new();
        for _ in 0..10 {
            if let Some(resp) = stream.next().await {
                if resp.is_err() {
                    continue;
                }
                let resp_packet = resp.unwrap();
                if let MqttPacket::PubRec(_, ack_pros) = resp_packet {
                    let user_properties = ack_pros.unwrap().user_properties;
                    if !user_properties.is_empty() {
                        let dump_flag = user_properties.first().unwrap();
                        if "PUBLISH_QOS_DUMP" == dump_flag.0 && "true" == dump_flag.1 {
                            resp_data.push(true);
                        }
                    }
                }
            }
        }

        assert!(!resp_data.is_empty());
    }

    #[tokio::test]
    async fn publish_qos2_pub_rel_dup_test() {
        let socket = TcpStream::connect("127.0.0.1:1883")
            .timeout(Duration::from_secs(3))
            .await
            .unwrap()
            .unwrap();
        let mut stream: Framed<TcpStream, Mqtt5Codec> = Framed::new(socket, Mqtt5Codec::new());
        let topic = format!("/publish_qos2_pub_rel_dup_test/{}/", unique_id(),);
        let message = "publish_qos2_pub_rel_dup_test mqtt message".to_string();

        // send connect package
        let packet = build_mqtt5_pg_connect();
        stream.send(packet).await.unwrap();
        stream.next().await;

        // publish publish data
        let pub_packet = MqttPacket::Publish(
            Publish {
                dup: false,
                qos: QoS::ExactlyOnce,
                p_kid: 1,
                retain: false,
                topic: Bytes::from(topic.to_string()),
                payload: Bytes::from(message.to_string()),
            },
            None,
        );
        stream.send(pub_packet.clone()).await.unwrap();
        let timeout = Duration::from_secs(60);
        let start = Instant::now();
        loop {
            if start.elapsed() >= timeout {
                panic!(
                    "publish_qos2_pub_rel_dup_test timeout waiting for PubRec after {} seconds",
                    timeout.as_secs()
                );
            }
            if let Some(resp) = stream.next().await {
                if resp.is_ok() {
                    let resp_packet = resp.unwrap();
                    if let MqttPacket::PubRec(_, _) = resp_packet {
                        let pub_rel_packet = MqttPacket::PubRel(
                            PubRel {
                                pkid: 1,
                                reason: Some(PubRelReason::Success),
                            },
                            None,
                        );
                        for _ in 0..10 {
                            stream.send(pub_rel_packet.clone()).await.unwrap();
                        }
                        let mut resp_data = Vec::new();
                        for _ in 0..10 {
                            if let Some(resp) = stream.next().await {
                                if resp.is_err() {
                                    continue;
                                }
                                let resp_packet = resp.unwrap();
                                if let MqttPacket::PubComp(ack, ack_pros) = resp_packet {
                                    if ack.reason.unwrap()
                                        == PubCompReason::PacketIdentifierNotFound
                                    {
                                        resp_data.push(true);
                                        continue;
                                    }

                                    if ack_pros.is_none() {
                                        continue;
                                    }
                                    let user_properties = ack_pros.unwrap().user_properties;
                                    if !user_properties.is_empty() {
                                        let dump_flag = user_properties.first().unwrap();
                                        if "PUBLISH_QOS_DUMP" == dump_flag.0
                                            && "true" == dump_flag.1
                                        {
                                            resp_data.push(true);
                                        }
                                    }
                                }
                            }
                        }

                        assert!(!resp_data.is_empty());
                        break;
                    }
                }
            }
        }
    }

    fn build_mqtt5_pg_connect() -> MqttPacket {
        let client_id = build_client_id("mqtt5_keep_alive_test");
        let login = Some(Login {
            username: "admin".to_string(),
            password: password(),
        });

        let connect: Connect = Connect {
            keep_alive: 5,
            client_id,
            clean_session: true,
        };

        MqttPacket::Connect(5, connect, None, None, None, login)
    }
}
