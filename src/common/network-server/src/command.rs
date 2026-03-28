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

use crate::common::packet::ResponsePackage;
use async_trait::async_trait;
use metadata_struct::connection::NetworkConnection;
use protocol::robust::RobustMQPacket;
use std::{net::SocketAddr, sync::Arc};

#[async_trait]
pub trait Command {
    async fn apply(
        &self,
        tcp_connection: &NetworkConnection,
        addr: &SocketAddr,
        packet: &RobustMQPacket,
    ) -> Option<ResponsePackage>;
}
pub type ArcCommandAdapter = Arc<Box<dyn Command + Send + Sync>>;

/// Routes incoming packets to the registered command handler for each protocol.
/// Each field corresponds to one protocol; `None` means that protocol is not
/// active on this node.
#[derive(Clone, Default)]
pub struct CommandRegistry {
    pub mqtt: Option<ArcCommandAdapter>,
    pub kafka: Option<ArcCommandAdapter>,
    pub amqp: Option<ArcCommandAdapter>,
    pub nats: Option<ArcCommandAdapter>,
    pub storage_engine: Option<ArcCommandAdapter>,
}

impl CommandRegistry {
    pub fn get(&self, packet: &RobustMQPacket) -> Option<&ArcCommandAdapter> {
        match packet {
            RobustMQPacket::MQTT(_) => self.mqtt.as_ref(),
            RobustMQPacket::KAFKA(_) => self.kafka.as_ref(),
            RobustMQPacket::AMQP(_) => self.amqp.as_ref(),
            RobustMQPacket::NATS(_) => self.nats.as_ref(),
            RobustMQPacket::StorageEngine(_) => self.storage_engine.as_ref(),
        }
    }
}
