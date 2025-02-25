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
    use mqtt_broker::server::quic::client::QuicClient;
    use mqtt_broker::server::quic::server::QuicServer;
    use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
    use quinn::{Connection, RecvStream, SendStream};
    use robustmq_test::mqtt_build_tool::build_connack::build_mqtt5_pg_connect_ack_wrapper;
    use robustmq_test::mqtt_build_tool::build_connect::build_mqtt5_pg_connect_wrapper;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;
    use tokio_util::codec::{Decoder, Encoder};

    #[tokio::test]
    async fn quic_client_should_connect_quic_server() {
        let ip_server: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        //
        // let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        // transport_config.max_concurrent_uni_streams(0_u8.into());

        let mut server = QuicServer::new(ip_server);

        server.start();

        let ip_server_addr = server.local_addr();

        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        tokio::spawn(async move {
            let conn = server.accept_connection().await.unwrap();
            assert_eq!(conn.remote_address(), client_addr);
        });

        let mut quic_client = QuicClient::bind(client_addr);
        let connection = quic_client.connect(ip_server_addr, "localhost");
        drop(connection);
        quic_client.wait_idle().await;
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
                .connect(ip_server_addr, "localhost")
                .await
                .unwrap();
            connection.closed().await;
        };

        let quic_client_2 = async move {
            let connection = quic_client_2
                .connect(ip_server_addr, "localhost")
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
        // initialization
        let mut server = create_server();
        server.start();
        let server_addr = server.local_addr();

        let mut quic_client = create_client();
        let client_addr = quic_client.local_addr();

        let server_recv = Arc::new(tokio::sync::Notify::new());
        let client_send = server_recv.clone();
        let client_recv = Arc::new(tokio::sync::Notify::new());
        let server_send = client_recv.clone();

        let server = tokio::spawn(async move {
            let conn = server.accept_connection().await.unwrap();

            server_recv.notified().await;
            let (mut server_send_stream, server_recv_stream) = conn.accept_bi().await.unwrap();
            receive_packet(server_recv_stream, Some(5)).await;
            let server_bytes_mut = build_bytes_mut(build_mqtt5_pg_connect_ack_wrapper, Some(5));
            send_packet(&mut server_send_stream, server_bytes_mut).await;

            server_send.notify_one();
        });

        let connection = quic_client.connect(server_addr, "localhost").await.unwrap();

        let (mut client_send_stream, client_recv_stream) = connection.open_bi().await.unwrap();

        let client_bytes_mut = build_bytes_mut(build_mqtt5_pg_connect_wrapper, Some(5));
        send_packet(&mut client_send_stream, client_bytes_mut).await;
        client_send.notify_one();

        client_recv.notified().await;
        receive_packet(client_recv_stream, Some(5)).await;

        server.await.unwrap();
    }

    async fn send_packet(send_stream: &mut SendStream, mut bytes_mut: BytesMut) {
        send_stream.write_all(bytes_mut.as_mut()).await.unwrap();
        send_stream.finish().unwrap();
        let _ = send_stream.stopped().await;
    }

    async fn receive_packet(mut recv_stream: RecvStream, protocol_version: Option<u8>) {
        let mut codec = MqttCodec::new(protocol_version);
        let mut decode_bytes = BytesMut::with_capacity(0);
        let vec = recv_stream.read_to_end(1024).await.unwrap();
        decode_bytes.extend(vec);
        let packet = codec.decode(&mut decode_bytes).unwrap();
        assert!(packet.is_some());
    }

    fn create_client() -> QuicClient {
        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        QuicClient::bind(client_addr)
    }

    fn create_server() -> QuicServer {
        let ip_server: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        QuicServer::new(ip_server)
    }

    async fn get_current_connection(server: QuicServer, client_addr: SocketAddr) -> Connection {
        let endpoint = server.get_endpoint().unwrap();
        let incoming_conn = endpoint.accept().await.unwrap();
        let conn = incoming_conn.await.unwrap();
        assert_eq!(conn.remote_address(), client_addr);
        conn
    }

    fn build_bytes_mut(
        create_connect_wrapper: fn() -> MqttPacketWrapper,
        protocol_version: Option<u8>,
    ) -> BytesMut {
        let mut mqtt_codec = MqttCodec::new(protocol_version);
        let connect_wrapper = create_connect_wrapper();

        let mut bytes_mut = BytesMut::with_capacity(0);
        mqtt_codec.encode(connect_wrapper, &mut bytes_mut).unwrap();
        bytes_mut
    }
}
