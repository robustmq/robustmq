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
    use crate::mqtt::protocol::quic_server::common::build_client_endpoint;
    use bytes::BytesMut;
    use network_server::quic::stream::{QuicFramedReadStream, QuicFramedWriteStream};
    use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
    use quinn::{ReadToEndError, VarInt};
    use robustmq_test::mqtt::protocol::build_connect::build_test_mqtt4_connect_packet_wrapper;
    use std::time::Duration;
    use tokio::io::AsyncReadExt;
    use tokio::time::timeout;
    use tokio_util::codec::Decoder;

    #[tokio::test]
    async fn quic_client_connect_test() {
        let client_endpoint = build_client_endpoint("0.0.0.0:0");

        let server_addr = "127.0.0.1:9083".parse().unwrap();

        let connection_result = timeout(Duration::from_secs(10), async {
            client_endpoint
                .connect(server_addr, "localhost")
                .unwrap()
                .await
        })
        .await;
        assert!(connection_result.is_ok());

        let connection = connection_result.unwrap().unwrap();
        connection.close(VarInt::from_u32(0), b"test completed");
        client_endpoint.wait_idle().await;

        let close_result = timeout(Duration::from_secs(10), connection.closed()).await;
        assert!(close_result.is_ok());
    }

    //test send packet  and receive ack
    #[tokio::test]
    async fn quic_client_send_packet_test() {
        let client_endpoint = build_client_endpoint("0.0.0.0:0");

        let server_addr = "127.0.0.1:9083".parse().unwrap();

        let connection = client_endpoint
            .connect(server_addr, "localhost")
            .unwrap()
            .await
            .expect("could not connect");

        let (send, _recv) = connection
            .open_bi()
            .await
            .expect("could not open bidirectional stream");

        let mqtt_4_wrapper = build_test_mqtt4_connect_packet_wrapper();

        let robust_mq_codec = RobustMQCodec::new();
        let mut write_stream = QuicFramedWriteStream::new(send, robust_mq_codec.clone());
        let mqtt_wrapper = RobustMQCodecWrapper::MQTT(mqtt_4_wrapper);
        // let mut read_stream = QuicFramedReadStream::new(recv, robust_mq_codec.clone());

        write_stream
            .send(mqtt_wrapper.clone())
            .await
            .expect("send failed");

        // while let result = recv.read_to_end(1024).await {
        //     match result {
        //         Ok(packet) => {
        //             let mut decode_bytes = BytesMut::with_capacity(0);
        //             decode_bytes.extend(packet);
        //
        //             if !decode_bytes.is_empty() {
        //                 let _decoded = robust_mq_codec.clone().decode_data(&mut decode_bytes);
        //             }
        //         }
        //         Err(_e) => {}
        //     }
        // }
        // use futures_util::StreamExt;
        // let _count = 0;

        // while let result = read_stream.receive().await {
        //     match result {
        //         Ok(packet) => {
        //             if let Some(pk) = packet {
        //                 println!("收到数据包: {:?}", pk);
        //                 count += 1;
        //                 if count >= 1 {
        //                     break; // 收到一个包就退出循环
        //                 }
        //             }
        //         }
        //         Err(e) => {
        //             println!("接收数据出错: {:?}", e);
        //             break;
        //         }
        //     }
        // }

        // connection.close(VarInt::from_u32(0), b"test completed");
        // client_endpoint.wait_idle().await;
    }
}
