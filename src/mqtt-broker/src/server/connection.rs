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

use std::fmt;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;

use log::error;
use protocol::mqtt::common::MQTTProtocol;
use tokio::sync::mpsc;
static CONNECTION_ID_BUILD: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, PartialEq, PartialOrd)]
pub enum NetworkConnectionType {
    TCP,
    TCPS,
    WebSocket,
    WebSockets,
}

impl fmt::Display for NetworkConnectionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NetworkConnectionType::TCP => "tcp",
                NetworkConnectionType::TCPS => "tcps",
                NetworkConnectionType::WebSocket => "websocket",
                NetworkConnectionType::WebSockets => "websockets",
            }
        )
    }
}

#[derive(Clone)]
pub struct NetworkConnection {
    pub connection_type: NetworkConnectionType,
    pub connection_id: u64,
    pub protocol: Option<MQTTProtocol>,
    pub addr: SocketAddr,
    pub connection_stop_sx: Option<mpsc::Sender<bool>>,
}

impl NetworkConnection {
    pub fn new(
        connection_type: NetworkConnectionType,
        addr: SocketAddr,
        connection_stop_sx: Option<mpsc::Sender<bool>>,
    ) -> Self {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        NetworkConnection {
            connection_type,
            connection_id,
            protocol: None,
            addr,
            connection_stop_sx,
        }
    }

    pub fn connection_id(&self) -> u64 {
        return self.connection_id;
    }

    pub fn set_protocol(&mut self, protocol: MQTTProtocol) {
        self.protocol = Some(protocol);
    }

    pub fn is_mqtt3(&self) -> bool {
        if let Some(protocol) = self.protocol.clone() {
            return protocol == MQTTProtocol::MQTT3;
        }
        return false;
    }

    pub fn is_mqtt4(&self) -> bool {
        if let Some(protocol) = self.protocol.clone() {
            return protocol == MQTTProtocol::MQTT4;
        }
        return false;
    }

    pub fn is_mqtt5(&self) -> bool {
        if let Some(protocol) = self.protocol.clone() {
            return protocol == MQTTProtocol::MQTT5;
        }
        return false;
    }

    pub fn is_tcp(&self) -> bool {
        return self.connection_type == NetworkConnectionType::TCP
            || self.connection_type == NetworkConnectionType::TCPS;
    }

    pub async fn stop_connection(&self) {
        if let Some(sx) = self.connection_stop_sx.clone() {
            match sx.send(true).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
    }
}
