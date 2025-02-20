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
use quinn::crypto::rustls::QuicClientConfig;
use quinn::ClientConfig;
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

#[allow(unused)]
pub fn make_server_endpoint(
    server_config: ServerConfig,
    bind_addr: SocketAddr,
) -> Result<Endpoint, Box<dyn Error + Send + Sync + 'static>> {
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok(endpoint)
}

pub struct QuicServerConfig {
    server_config: ServerConfig,
    bind_addr: SocketAddr,
}

impl QuicServerConfig {
    pub fn new(server_config: ServerConfig, bind_addr: SocketAddr) -> Self {
        QuicServerConfig {
            server_config,
            bind_addr,
        }
    }

    fn get_server_config(&self) -> ServerConfig {
        self.server_config.clone()
    }

    fn get_bind_addr(&self) -> SocketAddr {
        self.bind_addr.clone()
    }
}

pub struct QuicServer {
    quic_server_config: QuicServerConfig,
    endpoint: Option<Endpoint>,
}

impl QuicServer {
    pub fn new(quic_server_config: QuicServerConfig) -> Self {
        QuicServer {
            quic_server_config,
            endpoint: None,
        }
    }

    pub fn start(&mut self) -> Result<(), MqttBrokerError> {
        let endpoint = Endpoint::server(
            self.quic_server_config.get_server_config(),
            self.quic_server_config.get_bind_addr(),
        );
        match endpoint {
            Ok(endpoint) => {
                self.endpoint = Some(endpoint);
                Ok(())
            }
            Err(_) => Err(MqttBrokerError::CommonError(
                "Failed to start quic server".to_string(),
            )),
        }
    }

    pub fn get_endpoint(&self) -> Result<Endpoint, MqttBrokerError> {
        match &self.endpoint {
            Some(endpoint) => Ok(endpoint.clone()),
            None => Err(MqttBrokerError::CommonError(
                "Endpoint is not initialized".to_string(),
            )),
        }
    }
}
