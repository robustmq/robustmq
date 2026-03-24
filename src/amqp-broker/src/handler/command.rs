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

use amq_protocol::frame::AMQPFrame;
use amq_protocol::protocol::AMQPClass;
use async_trait::async_trait;
use metadata_struct::connection::NetworkConnection;
use network_server::command::{ArcCommandAdapter, Command};
use network_server::common::packet::ResponsePackage;
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::warn;

use crate::amqp::{basic, channel, connection, exchange, queue, tx};

pub fn create_command() -> ArcCommandAdapter {
    Arc::new(Box::new(AmqpHandlerCommand::new()))
}

#[derive(Clone)]
pub struct AmqpHandlerCommand {}

impl AmqpHandlerCommand {
    pub fn new() -> Self {
        AmqpHandlerCommand {}
    }
}

impl Default for AmqpHandlerCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Command for AmqpHandlerCommand {
    async fn apply(
        &self,
        _tcp_connection: &NetworkConnection,
        _addr: &SocketAddr,
        packet: &RobustMQPacket,
    ) -> Option<ResponsePackage> {
        match packet {
            RobustMQPacket::AMQP(frame) => {
                let resp_frame = self.process_frame(frame);
                resp_frame.map(|f| ResponsePackage {
                    connection_id: _tcp_connection.connection_id,
                    packet: RobustMQPacket::AMQP(f),
                })
            }
            _ => {
                warn!("AmqpHandlerCommand received non-AMQP packet");
                None
            }
        }
    }
}

impl AmqpHandlerCommand {
    fn process_frame(&self, frame: &AMQPFrame) -> Option<AMQPFrame> {
        match frame {
            AMQPFrame::Method(channel_id, class) => self.process_method(*channel_id, class),
            AMQPFrame::ProtocolHeader(_) => connection::process_protocol_header(),
            AMQPFrame::Heartbeat(channel_id) => connection::process_heartbeat(*channel_id),
            AMQPFrame::Header(channel_id, class_id, header) => {
                basic::process_header(*channel_id, *class_id, header)
            }
            AMQPFrame::Body(channel_id, data) => basic::process_body(*channel_id, data),
        }
    }

    fn process_method(&self, channel_id: u16, class: &AMQPClass) -> Option<AMQPFrame> {
        match class {
            // Connection class
            AMQPClass::Connection(method) => connection::process_connection(channel_id, method),
            // Channel class
            AMQPClass::Channel(method) => channel::process_channel(channel_id, method),
            // Exchange class
            AMQPClass::Exchange(method) => exchange::process_exchange(channel_id, method),
            // Queue class
            AMQPClass::Queue(method) => queue::process_queue(channel_id, method),
            // Basic class
            AMQPClass::Basic(method) => basic::process_basic(channel_id, method),
            // Tx class
            AMQPClass::Tx(method) => tx::process_tx(channel_id, method),
            // Access class (legacy, minimal support)
            AMQPClass::Access(_) => None,
            // Confirm class
            AMQPClass::Confirm(method) => basic::process_confirm(channel_id, method),
        }
    }
}
