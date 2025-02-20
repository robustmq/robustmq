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
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use rustls::Error;
use rustls_pki_types::PrivateKeyDer;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub fn generate_self_signed_cert() -> (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>) {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
    (vec![cert_der.clone()], priv_key.into())
}

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
        self.bind_addr.clone()
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
        match Endpoint::server(
            self.quic_server_config.get_server_config(),
            self.quic_server_config.get_bind_addr(),
        ) {
            Ok(endpoint) => {
                self.endpoint = Some(endpoint);
            }
            Err(_) => {
                panic!("Failed to start a quic server")
            }
        };
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
