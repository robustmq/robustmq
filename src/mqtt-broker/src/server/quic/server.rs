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

use crate::handler::cache::CacheManager;
use crate::handler::command::Command;
use crate::handler::error::MqttBrokerError;
use crate::security::AuthDriver;
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::{RequestPackage, ResponsePackage};
use crate::subscribe::subscribe_manager::SubscribeManager;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use delay_message::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use quinn::{Connection, Endpoint, ServerConfig, VarInt};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use rustls_pki_types::PrivateKeyDer;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::{broadcast, mpsc};

pub fn generate_self_signed_cert() -> (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>) {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
    (vec![cert_der.clone()], priv_key.into())
}

pub async fn start_quic_server<S>(
    subscribe_manager: Arc<SubscribeManager>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    message_storage_adapter: Arc<S>,
    delay_message_manager: Arc<DelayMessageManager<S>>,
    client_pool: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
    auth_driver: Arc<AuthDriver>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let conf = broker_mqtt_conf();
    let command = Command::new(
        cache_manager.clone(),
        message_storage_adapter.clone(),
        delay_message_manager.clone(),
        subscribe_manager.clone(),
        client_pool.clone(),
        connection_manager.clone(),
        auth_driver.clone(),
    );

    let mut server = QuicServer::new(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        conf.network.quic_port as u16,
    ));
    server.start();

    let quic_endpoint = server.get_endpoint();

    let (request_queue_sx, request_queue_rx) = mpsc::channel::<RequestPackage>(1000);
    let (response_queue_sx, response_queue_rx) = mpsc::channel::<ResponsePackage>(1000);

    let arc_quic_endpoint = Arc::new(quic_endpoint);
}

#[derive(Clone, Debug)]
pub struct QuicServerConfig {
    server_config: ServerConfig,
    bind_addr: SocketAddr,
}

impl QuicServerConfig {
    pub fn bind_addr(&mut self, addr: SocketAddr) {
        self.bind_addr = addr;
    }
    fn get_server_config(&self) -> ServerConfig {
        self.server_config.clone()
    }

    fn get_bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }
}

impl Default for QuicServerConfig {
    fn default() -> Self {
        let (cert_der, priv_key) = generate_self_signed_cert();
        let server_config = match ServerConfig::with_single_cert(cert_der, priv_key) {
            Ok(quin_server_config) => quin_server_config,
            Err(_) => {
                panic!("Failed to create quic server config in default")
            }
        };
        QuicServerConfig {
            server_config,
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
        }
    }
}

pub struct QuicServer {
    quic_server_config: QuicServerConfig,
    endpoint: Option<Endpoint>,
}

impl QuicServer {
    pub fn new(addr: SocketAddr) -> Self {
        let mut quinn_quic_server_config = QuicServerConfig::default();
        quinn_quic_server_config.bind_addr(addr);

        QuicServer {
            quic_server_config: quinn_quic_server_config,
            endpoint: None,
        }
    }

    pub fn start(&mut self) {
        let endpoint = self.create_quinn_endpoint_as_a_quic_server();
        self.bind_address_for_quic_server_config(endpoint);
    }

    fn bind_address_for_quic_server_config(&mut self, endpoint: Endpoint) {
        let local_addr = match endpoint.local_addr() {
            Ok(local_addr) => local_addr,
            Err(e) => {
                panic!("we can not to bind this address: {}", e)
            }
        };

        self.quic_server_config.bind_addr = local_addr;
    }

    fn create_quinn_endpoint_as_a_quic_server(&mut self) -> Endpoint {
        match Endpoint::server(
            self.quic_server_config.get_server_config(),
            self.quic_server_config.get_bind_addr(),
        ) {
            Ok(endpoint) => {
                self.endpoint = Some(endpoint.clone());
                endpoint
            }
            Err(e) => {
                panic!("Failed to start a quic server: {}", e)
            }
        }
    }

    pub fn get_endpoint(&self) -> Endpoint {
        match &self.endpoint {
            Some(endpoint) => endpoint.clone(),
            None => {
                panic!("quic server is not initialized")
            }
        }
    }

    pub async fn accept_connection(&self) -> Result<Connection, MqttBrokerError> {
        if self.endpoint.as_ref().is_none() {
            return Err(MqttBrokerError::CommonError(
                "Server is not initialized".to_string(),
            ));
        }

        let incoming = self.endpoint.as_ref().unwrap().accept().await;

        if incoming.is_none() {
            return Err(MqttBrokerError::CommonError(
                "No incoming connection".to_string(),
            ));
        }

        match incoming.unwrap().await {
            Ok(connection) => Ok(connection),
            Err(e) => Err(MqttBrokerError::CommonError(format!(
                "Failed to accept connection: {}",
                e
            ))),
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        match &self.endpoint {
            Some(endpoint) => endpoint.local_addr().unwrap(),
            None => panic!("quic server is not initialized"),
        }
    }

    pub fn close(&mut self, error_code: VarInt, reason: &[u8]) {
        match &self.endpoint {
            Some(endpoint) => endpoint.close(error_code, reason),
            None => {
                panic!("quic server is not initialized")
            }
        }
    }

    pub async fn wait_idle(&self) {
        match &self.endpoint {
            None => {
                panic!("quic server is not initialized")
            }
            Some(endpoint) => {
                endpoint.wait_idle().await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::matchers::{anything, none};
    use googletest::{assert_that, gtest};

    #[gtest]
    #[tokio::test]
    async fn should_create_quic_server() {
        let quic_server = QuicServer::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0));

        assert_that!(quic_server.endpoint, none());

        assert_that!(quic_server.quic_server_config, anything());
    }

    #[gtest]
    #[tokio::test]
    async fn should_start_quic_server() {
        let mut quic_server =
            QuicServer::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080));

        assert_that!(quic_server.endpoint, none());

        quic_server.start();

        assert_that!(quic_server.endpoint, anything());
    }
}
