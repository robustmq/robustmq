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

use quinn::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use mqtt_broker::server::quic::client::run_client;
    use mqtt_broker::server::quic::server::{QuicServer, QuicServerConfig};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    // todo create a client to connect to server
    // todo server need to encapuslate
    #[tokio::test]
    async fn quic_client_should_connect_quic_server() {
        let ip_server: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_der = CertificateDer::from(cert.cert);
        let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

        let mut server_config =
            ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into()).unwrap();
        let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        transport_config.max_concurrent_uni_streams(0_u8.into());

        let config = QuicServerConfig::new(server_config, ip_server);
        let mut server = QuicServer::new(config);

        server.start().expect("TODO: panic message");

        tokio::spawn(async move {
            let endpoint = server.get_endpoint().unwrap();
            let incoming_conn = endpoint.accept().await.unwrap();
            let conn = incoming_conn.await.unwrap();
            println!(
                "[server] connection accepted: addr={}",
                conn.remote_address()
            );
        });

        run_client(ip_server).await.unwrap();
    }

    // todo server has a accept function to get a connection
}
