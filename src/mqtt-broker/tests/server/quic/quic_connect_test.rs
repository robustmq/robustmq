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
    use mqtt_broker::server::quic::server::QuicServer;
    use network_server::quic::stream::QuicFramedWriteStream;
    use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
    use robustmq_test::mqtt_build_tool::build_connack::build_mqtt5_pg_connect_ack_wrapper;
    use robustmq_test::mqtt_build_tool::build_connect::build_mqtt5_pg_connect_wrapper;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use crate::server::quic::{client::QuicClient, quic_common::set_up};
    use protocol::mqtt::common::MqttPacket;
    use std::sync::Arc;
    use tokio_util::codec::{Decoder, Encoder};

    #[tokio::test]
    async fn quic_client_should_connect_quic_server() {
        let (server, mut client) = set_up().await;

        let ip_server_addr = server.local_addr();

        let ip_client_addr = client.local_addr();
        tokio::spawn(async move {
            let conn = server.accept_connection().await.unwrap();
            assert_eq!(conn.remote_address(), ip_client_addr);
        });

        let connection = client.connect(ip_server_addr, "127.0.0.1");
        drop(connection);
        client.wait_idle().await;
    }

    #[tokio::test]
    async fn quic_server_should_receive_data_from_different_client() {
        let ip_server: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        //
        // let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        // transport_config.max_concurrent_uni_streams(0_u8.into());

        let mut server = QuicServer::new(ip_server);

        server.start();
        let ip_server_addr = server.local_addr();

        let client_addr_1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let client_addr_2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        let mut quic_client_1 = QuicClient::bind(client_addr_1);
        let mut quic_client_2 = QuicClient::bind(client_addr_2);

        let client_addr_1 = quic_client_1.local_addr();
        let client_addr_2 = quic_client_2.local_addr();

        let quic_client_1 = async move {
            let connection = quic_client_1
                .connect(ip_server_addr, "127.0.0.1")
                .await
                .unwrap();
            connection.closed().await;
        };

        let quic_client_2 = async move {
            let connection = quic_client_2
                .connect(ip_server_addr, "127.0.0.1")
                .await
                .unwrap();
            connection.closed().await;
        };

        tokio::spawn(async move {
            while let Ok(connection) = server.accept_connection().await {
                assert!(
                    connection.remote_address() == client_addr_1
                        || connection.remote_address() == client_addr_2
                );
                connection.close(42u32.into(), &[]);
            }
        });

        tokio::join!(quic_client_1, quic_client_2);
    }

    #[tokio::test]
    async fn each_receive_and_send_data() {
        let (server, mut client) = set_up().await;
        let server_addr = server.local_addr();
        let server_recv = Arc::new(tokio::sync::Notify::new());
        let client_send = server_recv.clone();
        let client_recv = Arc::new(tokio::sync::Notify::new());
        let server_send = client_recv.clone();

        let _verify_mqtt_packet =
            construct_verify_mqtt_packet(build_mqtt5_pg_connect_wrapper, Some(5));

        let _server = tokio::spawn(async move {
            let conn = server.accept_connection().await.unwrap();
            server_recv.notified().await;
            let (server_send_stream, _server_recv_stream) = conn.accept_bi().await.unwrap();
            // receive_packet(server_recv_stream, Some(5)).await;
            // let mut quic_framed_read_stream =
            //     QuicMQTTFramedWriteStream::new(server_recv_stream, MqttCodec::new(Some(5)));
            // match quic_framed_read_stream.receive().await {
            //     Ok(packet) => {
            //         assert_eq!(packet.unwrap(), verify_mqtt_packet)
            //     }
            //     Err(_) => {
            //         unreachable!()
            //     }
            // }
            let mut quic_framed_write_stream =
                QuicFramedWriteStream::new(server_send_stream, MqttCodec::new(Some(5)));
            if let Err(_e) = quic_framed_write_stream
                .send(build_mqtt5_pg_connect_ack_wrapper())
                .await
            {
                unreachable!()
            }
            server_send.notify_one();
        });

        let connection = client.connect(server_addr, "127.0.0.1").await.unwrap();

        let (client_send_stream, _client_recv_stream) = connection.open_bi().await.unwrap();
        if let Err(_e) = QuicFramedWriteStream::new(client_send_stream, MqttCodec::new(None))
            .send(build_mqtt5_pg_connect_wrapper())
            .await
        {
            unreachable!()
        }
        client_send.notify_one();

        client_recv.notified().await;
        // prepare to receive packet
        let _verify_mqtt_packet =
            construct_verify_mqtt_packet(build_mqtt5_pg_connect_ack_wrapper, Some(5));

        // act
        // let mut quic_framed_read_stream =
        //     QuicMQTTFramedWriteStream::new(client_send_stream, MqttCodec::new(Some(5)));
        // match quic_framed_read_stream.receive().await {
        //     Ok(packet) => {
        //         // verify receive packet
        //         assert_eq!(packet.unwrap(), verify_mqtt_packet)
        //     }
        //     Err(_) => {
        //         unreachable!()
        //     }
        // }

        // server.await.unwrap();
    }

    fn construct_verify_mqtt_packet(
        create_mqtt_packet_wrapper: fn() -> MqttPacketWrapper,
        protocol_version: Option<u8>,
    ) -> MqttPacket {
        let mut codec = MqttCodec::new(protocol_version);
        let mut test_bytes_mut = build_bytes_mut(create_mqtt_packet_wrapper, protocol_version);
        codec.decode(&mut test_bytes_mut).unwrap().unwrap()
    }

    fn build_bytes_mut(
        create_mqtt_packet_wrapper: fn() -> MqttPacketWrapper,
        protocol_version: Option<u8>,
    ) -> BytesMut {
        let mut mqtt_codec = MqttCodec::new(protocol_version);
        let connect_wrapper = create_mqtt_packet_wrapper();

        let mut bytes_mut = BytesMut::with_capacity(0);
        mqtt_codec.encode(connect_wrapper, &mut bytes_mut).unwrap();
        bytes_mut
    }
}
