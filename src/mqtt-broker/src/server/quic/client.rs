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

use crate::handler::error::MqttBrokerError;
use crate::server::quic::skip_server_verification::SkipServerVerification;
use log::info;
use quinn::{ClientConfig, Connection, Endpoint};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

pub async fn run_client(
    server_addr: SocketAddr,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let mut endpoint = Endpoint::client(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8082))?;

    endpoint.set_default_client_config(ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(
            rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(SkipServerVerification::new())
                .with_no_client_auth(),
        )?,
    )));

    // connect to server
    let connection: Connection = endpoint
        .connect(server_addr, "localhost")
        .unwrap()
        .await
        .unwrap();
    println!("[client] connected: addr={}", connection.remote_address());
    // Dropping handles allows the corresponding objects to automatically shut down
    drop(connection);
    // Make sure the server has a chance to clean up
    endpoint.wait_idle().await;

    Ok(())
}

#[derive(Clone)]
struct QuicClientConfig {
    quic_client_config: ClientConfig,
    bind_addr: SocketAddr,
}

impl QuicClientConfig {
    pub fn get_quic_client_config(&self) -> ClientConfig {
        self.quic_client_config.clone()
    }

    pub fn get_bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    pub fn bind_addr(&mut self, bind_addr: SocketAddr) {
        self.bind_addr = bind_addr;
    }
}

impl Default for QuicClientConfig {
    fn default() -> Self {
        let quic_client_config = ClientConfig::new(create_default_crypto());
        Self {
            quic_client_config,
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        }
    }
}

fn create_default_crypto() -> Arc<quinn::crypto::rustls::QuicClientConfig> {
    let crypto = match quinn::crypto::rustls::QuicClientConfig::try_from(
        rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(SkipServerVerification::new())
            .with_no_client_auth(),
    ) {
        Ok(quic_client_config) => Arc::new(quic_client_config),
        Err(_) => panic!("failed to create quic client config"),
    };
    crypto
}

pub struct QuicClient {
    quic_client_config: QuicClientConfig,
    endpoint: Option<Endpoint>,
}

impl QuicClient {
    pub fn bind(addr: SocketAddr) -> Self {
        let mut quinn_quic_client_config = QuicClientConfig::default();
        quinn_quic_client_config.bind_addr(addr);

        match Endpoint::client(quinn_quic_client_config.get_bind_addr()) {
            Ok(endpoint) => Self {
                quic_client_config: quinn_quic_client_config,
                endpoint: Some(endpoint),
            },
            Err(_) => panic!("failed to bind ip in quic client"),
        }
    }
    fn get_endpoint(&self) -> Result<Endpoint, MqttBrokerError> {
        let endpoint = match &self.endpoint {
            None => {
                return Err(MqttBrokerError::CommonError(
                    "quic client endpoint is not initialized".to_string(),
                ))
            }
            Some(endpoint) => endpoint.clone(),
        };
        Ok(endpoint)
    }

    pub async fn connect(
        &mut self,
        server_addr: SocketAddr,
        server_name: &str,
    ) -> Result<Connection, MqttBrokerError> {
        let endpoint = self.get_endpoint()?;

        let connecting = match endpoint.connect(server_addr, server_name) {
            Ok(connecting) => connecting,
            Err(e) => {
                return Err(MqttBrokerError::CommonError(format!(
                    "failed to connect quic server, error: {:?}",
                    e
                )))
            }
        };

        let connection = match connecting.await {
            Ok(connection) => connection,
            Err(e) => {
                return Err(MqttBrokerError::CommonError(format!(
                    "can not get a connection, error: {:?}",
                    e
                )))
            }
        };

        info!("[client] connected: addr={}", connection.remote_address());

        Ok(connection)
    }

    pub async fn disconnect(&mut self) {
        match self.get_endpoint() {
            Ok(endpoint) => {
                endpoint.wait_idle().await;
                self.endpoint = None;
            }
            Err(_) => {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::server::quic::client::{QuicClient, QuicClientConfig};
    use std::net::SocketAddr;
    #[tokio::test]
    async fn should_create_a_quic_client_config_ip_default() {
        let client_ip: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let mut config = QuicClientConfig::default();
        config.bind_addr(client_ip);
        assert_eq!(config.get_bind_addr(), client_ip);
    }

    #[tokio::test]
    async fn should_create_a_quic_client_by_config() {
        let client_ip: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let quic_client = QuicClient::bind(client_ip);
        assert!(quic_client.endpoint.is_some());
    }

    // #[tokio::test]
    // fn should_create_a_quic_client() {
    //     let client_ip: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    //     let quic_client = QuicClient::new(client_ip);
    //     assert!(quic_client.is_ok());
    // }
}
