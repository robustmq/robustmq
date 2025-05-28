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

use mqtt_broker::server::quic::client::QuicClient;
use mqtt_broker::server::quic::server::QuicServer;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn create_client() -> QuicClient {
    let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
    QuicClient::bind(client_addr)
}

fn create_server() -> QuicServer {
    let ip_server: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

    let mut server = QuicServer::new(ip_server);

    server.start();

    server
}

pub async fn set_up() -> (QuicServer, QuicClient) {
    let server = create_server();
    let client = create_client();
    (server, client)
}
