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

use crate::nats::{connect, ping, publish, subscribe};
use async_trait::async_trait;
use metadata_struct::connection::NetworkConnection;
use network_server::command::Command;
use network_server::common::packet::ResponsePackage;
use protocol::nats::packet::NatsPacket;
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct NatsHandlerCommand {}

impl NatsHandlerCommand {
    pub fn new() -> Self {
        NatsHandlerCommand {}
    }
}

impl Default for NatsHandlerCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Command for NatsHandlerCommand {
    async fn apply(
        &self,
        tcp_connection: &NetworkConnection,
        _addr: &SocketAddr,
        robust_packet: &RobustMQPacket,
    ) -> Option<ResponsePackage> {
        let packet = robust_packet.get_nats_packet()?;
        let connection_id = tcp_connection.connection_id;

        let resp_packet = match &packet {
            NatsPacket::Connect(req) => connect::process_connect(req),
            NatsPacket::Pub {
                subject,
                reply_to,
                payload,
            } => publish::process_pub(subject, reply_to.as_deref(), payload),
            NatsPacket::Sub {
                subject,
                queue_group,
                sid,
            } => subscribe::process_sub(subject, queue_group.as_deref(), sid),
            NatsPacket::Unsub { sid, max_msgs } => subscribe::process_unsub(sid, *max_msgs),
            NatsPacket::Ping => ping::process_ping(),
            NatsPacket::Pong => ping::process_pong(),
            // Server-to-client packets; not expected from a client
            NatsPacket::Info(_) | NatsPacket::Msg { .. } | NatsPacket::Ok | NatsPacket::Err(_) => {
                None
            }
        }?;

        Some(ResponsePackage::new(
            connection_id,
            RobustMQPacket::NATS(resp_packet),
        ))
    }
}

pub fn create_command() -> Arc<Box<dyn Command + Send + Sync>> {
    Arc::new(Box::new(NatsHandlerCommand::new()))
}
