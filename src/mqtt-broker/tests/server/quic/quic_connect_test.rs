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
    use protocol::mqtt::codec::MqttCodec;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;
    use tokio_util::codec::Encoder;

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
            let endpoint = server.get_endpoint().unwrap();
            let incoming_conn = endpoint.accept().await.unwrap();
            let conn = incoming_conn.await.unwrap();
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

        let endpoint = server.get_endpoint().unwrap();

        tokio::spawn(async move {
            while let Some(conn) = endpoint.accept().await {
                let connection = conn.await.unwrap();
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
    async fn quic_client_receive_data_and_quic_server_sent_data() {
        let ip_server: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        let mut server = QuicServer::new(ip_server);
        server.start();

        let ip_server_addr = server.local_addr();

        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        let mut quic_client = QuicClient::bind(client_addr);

        let client_addr = quic_client.local_addr();

        const MSG: &[u8; 5] = b"hello";

        let server = tokio::spawn(async move {
            let endpoint = server.get_endpoint().unwrap();
            let incoming_conn = endpoint.accept().await.unwrap();
            let conn = incoming_conn.await.unwrap();
            assert_eq!(conn.remote_address(), client_addr);
            let (mut send_stream, _recv_stream) = conn.open_bi().await.unwrap();
            send_stream.write_all(MSG).await.unwrap();
            send_stream.finish().unwrap();
            let _ = send_stream.stopped().await;
        });

        let connection = quic_client
            .connect(ip_server_addr, "localhost")
            .await
            .unwrap();
        let (_send_stream, mut recv_stream) = connection.accept_bi().await.unwrap();
        assert_eq!(recv_stream.read_to_end(MSG.len()).await.unwrap(), MSG);

        server.await.unwrap();
    }

    #[tokio::test]
    async fn quic_client_send_data_and_quic_server_receive_data() {
        let ip_server: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        let mut server = QuicServer::new(ip_server);
        server.start();

        let ip_server_addr = server.local_addr();

        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        let mut quic_client = QuicClient::bind(client_addr);

        let client_addr = quic_client.local_addr();

        const MSG: &[u8; 5] = b"hello";

        let write_send = Arc::new(tokio::sync::Notify::new());
        let write_recv = write_send.clone();

        let server = tokio::spawn(async move {
            let endpoint = server.get_endpoint().unwrap();
            let incoming_conn = endpoint.accept().await.unwrap();
            let conn = incoming_conn.await.unwrap();
            assert_eq!(conn.remote_address(), client_addr);
            write_recv.notified().await;
            let (_send_stream, mut recv_stream) = conn.accept_bi().await.unwrap();
            assert_eq!(recv_stream.read_to_end(MSG.len()).await.unwrap(), MSG);
        });

        let connection = quic_client
            .connect(ip_server_addr, "localhost")
            .await
            .unwrap();
        let (mut send_stream, _recv_stream) = connection.open_bi().await.unwrap();
        send_stream.write_all(MSG).await.unwrap();
        send_stream.finish().unwrap();
        let _ = send_stream.stopped().await;
        write_send.notify_one();

        server.await.unwrap();
    }

    #[tokio::test]
    async fn each_receive_and_send_data() {
        let ip_server: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        let mut server = QuicServer::new(ip_server);

        server.start();

        let ip_server_addr = server.local_addr();

        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        let mut quic_client = QuicClient::bind(client_addr);

        let client_addr = quic_client.local_addr();

        const SENDMSG: &[u8; 14] = b"hello, server!";
        const RESPONSE: &[u8; 14] = b"hello, client!";

        let server_recv = Arc::new(tokio::sync::Notify::new());
        let client_send = server_recv.clone();
        let client_recv = Arc::new(tokio::sync::Notify::new());
        let server_send = client_recv.clone();

        let server = tokio::spawn(async move {
            let endpoint = server.get_endpoint().unwrap();
            let incoming_conn = endpoint.accept().await.unwrap();
            let conn = incoming_conn.await.unwrap();
            assert_eq!(conn.remote_address(), client_addr);
            server_recv.notified().await;
            let (mut send_stream, mut recv_stream) = conn.accept_bi().await.unwrap();
            assert_eq!(
                recv_stream.read_to_end(SENDMSG.len()).await.unwrap(),
                SENDMSG
            );
            send_stream.write_all(RESPONSE).await.unwrap();
            send_stream.finish().unwrap();
            let _ = send_stream.stopped().await;
            server_send.notify_one();
        });

        let connection = quic_client
            .connect(ip_server_addr, "localhost")
            .await
            .unwrap();
        let (mut send_stream, mut recv_stream) = connection.open_bi().await.unwrap();
        send_stream.write_all(SENDMSG).await.unwrap();
        send_stream.finish().unwrap();
        let _ = send_stream.stopped().await;
        client_send.notify_one();
        client_recv.notified().await;
        assert_eq!(
            recv_stream.read_to_end(RESPONSE.len()).await.unwrap(),
            RESPONSE
        );

        server.await.unwrap();
    }

    #[tokio::test]
    async fn try_encode_data_from_mqtt_encoder() {
        const MSG: &[u8; 5] = b"hello";
        let bytes_mut = BytesMut::with_capacity(MSG.len());
        let mqtt_codec = MqttCodec::new(None);
        mqtt_codec.encode()
    }
}
