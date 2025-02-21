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
    use mqtt_broker::server::quic::client::QuicClient;
    use mqtt_broker::server::quic::server::QuicServer;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn quic_client_should_connect_quic_server() {
        let ip_server: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

        //
        // let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        // transport_config.max_concurrent_uni_streams(0_u8.into());

        let mut server = QuicServer::new(ip_server);

        server.start();

        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8082);

        tokio::spawn(async move {
            let endpoint = server.get_endpoint().unwrap();
            let incoming_conn = endpoint.accept().await.unwrap();
            let conn = incoming_conn.await.unwrap();
            assert_eq!(conn.remote_address(), client_addr);
        });

        let mut quic_client = QuicClient::bind(client_addr);
        let connection = quic_client.connect(ip_server, "localhost");
        drop(connection);
        quic_client.wait_idle().await;
    }

    // todo server can receive data twice from different clients
    #[tokio::test]
    #[ignore]
    async fn quic_server_should_receive_data_from_different_client() {
        let ip_server: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

        //
        // let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        // transport_config.max_concurrent_uni_streams(0_u8.into());

        let mut server = QuicServer::new(ip_server);

        server.start();

        let client_addr_1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8081);
        let client_addr_2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8082);

        let mut quic_client_1 = QuicClient::bind(client_addr_1);
        let mut quic_client_2 = QuicClient::bind(client_addr_2);

        let quic_client_1 = async move {
            let connection = quic_client_1.connect(ip_server, "localhost").await.unwrap();
            connection.closed().await;
        };
    }
}
