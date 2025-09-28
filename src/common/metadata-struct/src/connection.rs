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

use common_base::tools::now_second;
use protocol::mqtt::common::MqttProtocol;
use protocol::robust::RobustMQProtocol;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;
use tokio::sync::mpsc;
use tracing::debug;
static CONNECTION_ID_BUILD: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum NetworkConnectionType {
    Tcp,
    Tls,
    WebSocket,
    WebSockets,
    QUIC,
}

impl fmt::Display for NetworkConnectionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NetworkConnectionType::Tcp => "Tcp",
                NetworkConnectionType::Tls => "Tls",
                NetworkConnectionType::WebSocket => "Websocket",
                NetworkConnectionType::WebSockets => "Websockets",
                NetworkConnectionType::QUIC => "Quic",
            }
        )
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub connection_type: NetworkConnectionType,
    pub connection_id: u64,
    pub protocol: Option<RobustMQProtocol>,
    pub addr: SocketAddr,
    pub create_time: u64,
    #[serde(skip_serializing, skip_deserializing)]
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
            create_time: now_second(),
            connection_stop_sx,
        }
    }

    pub fn connection_id(&self) -> u64 {
        self.connection_id
    }

    pub fn set_protocol(&mut self, protocol: RobustMQProtocol) {
        self.protocol = Some(protocol);
    }

    pub fn is_mqtt3(&self) -> bool {
        if let Some(protocol) = self.protocol.clone() {
            return protocol == RobustMQProtocol::MQTT3;
        }
        false
    }

    pub fn is_mqtt4(&self) -> bool {
        if let Some(protocol) = self.protocol.clone() {
            return protocol == RobustMQProtocol::MQTT4;
        }
        false
    }

    pub fn is_mqtt5(&self) -> bool {
        if let Some(protocol) = self.protocol.clone() {
            return protocol == RobustMQProtocol::MQTT5;
        }
        false
    }

    pub fn get_protocol(&self) -> MqttProtocol {
        if self.is_mqtt3() {
            return MqttProtocol::Mqtt3;
        }

        if self.is_mqtt4() {
            return MqttProtocol::Mqtt4;
        }

        MqttProtocol::Mqtt5
    }

    pub fn is_tcp(&self) -> bool {
        self.connection_type == NetworkConnectionType::Tcp
            || self.connection_type == NetworkConnectionType::Tls
    }

    pub fn is_quic(&self) -> bool {
        self.connection_type == NetworkConnectionType::QUIC
    }

    pub async fn stop_connection(&self) {
        if let Some(sx) = self.connection_stop_sx.clone() {
            if let Err(e) = sx.send(true).await {
                debug!("{}", e);
            }
        }
    }
}

pub fn calc_child_channel_index(index: usize, len: usize) -> usize {
    let seq = index % len;
    if seq == 0 {
        return 1;
    }
    seq
}
