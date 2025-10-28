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
    use bytes::BytesMut;
    use futures::{SinkExt, StreamExt};
    use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
    use protocol::mqtt::mqttv4::codec::Mqtt4Codec;
    use protocol::mqtt::mqttv5::codec::Mqtt5Codec;

    use robustmq_test::mqtt::protocol::build_connack::build_mqtt5_pg_connect_ack_wrapper;
    use robustmq_test::mqtt::protocol::build_connect::{
        build_mqtt4_connect_packet, build_mqtt5_pg_connect,
    };
    use std::time::Duration;
    use tokio::io;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::sleep;
    use tokio_util::codec::{Decoder, Encoder, Framed, FramedRead, FramedWrite};

    #[tokio::test]
    async fn try_encode_data_from_mqtt_encoder() {
        let mut mqtt_codec = MqttCodec::new(None);
        let connect = build_mqtt4_connect_packet();
        let mqtt_packet_wrapper = MqttPacketWrapper {
            protocol_version: 4,
            packet: connect,
        };
        let mut bytes_mut = BytesMut::with_capacity(0);
        mqtt_codec
            .encode(mqtt_packet_wrapper, &mut bytes_mut)
            .unwrap();
        assert!(!bytes_mut.is_empty());
    }

    #[tokio::test]
    async fn try_decode_data_from_mqtt_decoder() {
        let mut mqtt_codec = MqttCodec::new(None);
        let connect = build_mqtt4_connect_packet();
        let mqtt_packet_wrapper = MqttPacketWrapper {
            protocol_version: 4,
            packet: connect,
        };
        let mut bytes_mut = BytesMut::with_capacity(0);
        mqtt_codec
            .encode(mqtt_packet_wrapper, &mut bytes_mut)
            .unwrap();
        assert!(!bytes_mut.is_empty());
        let packet = mqtt_codec.decode(&mut bytes_mut).unwrap();
        assert!(packet.is_some());
    }

    #[tokio::test]

    async fn mqtt_frame_server() {
        let req_packet = build_mqtt4_connect_packet();
        let resp_packet = build_mqtt5_pg_connect_ack_wrapper();

        let resp = resp_packet.clone();
        tokio::spawn(async move {
            let ip = "127.0.0.1:1885";
            let listener = TcpListener::bind(ip).await.unwrap();
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let (r_stream, w_stream) = io::split(stream);
                let codec = MqttCodec::new(None);
                let mut read_frame_stream = FramedRead::new(r_stream, codec.clone());
                let mut write_frame_stream = FramedWrite::new(w_stream, codec.clone());
                while let Some(Ok(data)) = read_frame_stream.next().await {
                    println!("Got: {data:?}");

                    write_frame_stream.send(resp.clone()).await.unwrap();
                }
            }
        });

        sleep(Duration::from_secs(5)).await;

        // mqtt4
        let socket = TcpStream::connect("127.0.0.1:1885").await.unwrap();
        let mut stream: Framed<TcpStream, Mqtt4Codec> = Framed::new(socket, Mqtt4Codec::new());

        // send connect package
        let _ = stream.send(req_packet.clone()).await;

        if let Some(data) = stream.next().await {
            match data {
                Ok(da) => {
                    //assert_eq!(da, resp_packet.packet)
                    println!("{da:?}");
                }
                Err(e) => {
                    panic!("error: {e:?}");
                }
            }
        }

        let socket = TcpStream::connect("127.0.0.1:1885").await.unwrap();
        let mut stream: Framed<TcpStream, Mqtt5Codec> = Framed::new(socket, Mqtt5Codec::new());

        // send connect package
        let packet = build_mqtt5_pg_connect();
        let _ = stream.send(packet).await;

        if let Some(data) = stream.next().await {
            match data {
                Ok(da) => {
                    //assert_eq!(da, resp_packet.packet)
                    println!("{da:?}");
                }
                Err(e) => {
                    panic!("error: {e:?}");
                }
            }
        }
    }
}
