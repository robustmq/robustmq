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

use std::sync::Arc;

use amq_protocol::frame::AMQPFrame;
use async_trait::async_trait;
use common_security::manager::SecurityManager;
use metadata_struct::connection::NetworkConnection;
use network_server::command::{ArcCommandAdapter, Command};
use network_server::common::connection_manager::ConnectionManager;
use network_server::common::packet::ResponsePackage;
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;
use storage_adapter::driver::StorageDriverManager;
use tracing::{debug, warn};

use crate::amqp::basic::BasicCtx;
use crate::amqp::{basic, channel, connection, exchange, queue, tx};
use crate::core::cache::AmqpCacheManager;
use crate::core::connection::AmqpConnection;

pub fn create_command_with_state(
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    amqp_cache: Arc<AmqpCacheManager>,
    security_manager: Arc<SecurityManager>,
) -> ArcCommandAdapter {
    Arc::new(Box::new(AmqpHandlerCommand::new(
        connection_manager,
        storage_driver_manager,
        amqp_cache,
        security_manager,
    )))
}

#[derive(Clone)]
pub struct AmqpHandlerCommand {
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    amqp_cache: Arc<AmqpCacheManager>,
    security_manager: Arc<SecurityManager>,
}

impl AmqpHandlerCommand {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        amqp_cache: Arc<AmqpCacheManager>,
        security_manager: Arc<SecurityManager>,
    ) -> Self {
        AmqpHandlerCommand {
            connection_manager,
            storage_driver_manager,
            amqp_cache,
            security_manager,
        }
    }

    fn basic_ctx(&self) -> BasicCtx {
        BasicCtx {
            connection_manager: self.connection_manager.clone(),
            storage_driver_manager: self.storage_driver_manager.clone(),
            amqp_cache: self.amqp_cache.clone(),
        }
    }
}

#[async_trait]
impl Command for AmqpHandlerCommand {
    async fn apply(
        &self,
        tcp_connection: &NetworkConnection,
        _addr: &SocketAddr,
        packet: &RobustMQPacket,
    ) -> Option<ResponsePackage> {
        match packet {
            RobustMQPacket::AMQP(frame) => {
                let connection_id = tcp_connection.connection_id;
                let resp_frame = self.process_frame(frame, connection_id).await;
                resp_frame.map(|f| ResponsePackage {
                    connection_id,
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
    async fn process_frame(&self, frame: &AMQPFrame, connection_id: u64) -> Option<AMQPFrame> {
        let result = match frame {
            AMQPFrame::Method(channel_id, class) => {
                self.process_method(*channel_id, class, connection_id).await
            }
            AMQPFrame::ProtocolHeader(_) => {
                self.amqp_cache
                    .set_connection(AmqpConnection::new(connection_id));
                connection::process_protocol_header()
            }
            AMQPFrame::Heartbeat(channel_id) => connection::process_heartbeat(*channel_id),
            AMQPFrame::Header(channel_id, class_id, header) => {
                basic::process_content_header_full(
                    connection_id,
                    *channel_id,
                    *class_id,
                    header,
                    &self.basic_ctx(),
                )
                .await
            }
            AMQPFrame::Body(channel_id, data) => {
                basic::process_content_body_full(
                    connection_id,
                    *channel_id,
                    data,
                    &self.basic_ctx(),
                )
                .await
            }
        };
        if result.is_none() {
            debug!("AMQP frame has no response: {:?}", frame);
        }
        result
    }

    async fn process_method(
        &self,
        channel_id: u16,
        class: &amq_protocol::protocol::AMQPClass,
        connection_id: u64,
    ) -> Option<AMQPFrame> {
        use amq_protocol::protocol::AMQPClass;
        let result = match class {
            AMQPClass::Connection(method) => {
                use amq_protocol::protocol::connection::AMQPMethod as ConnMethod;
                let is_close = matches!(method, ConnMethod::Close(_) | ConnMethod::CloseOk(_));
                let frame = connection::process_connection_full(
                    channel_id,
                    method,
                    connection_id,
                    &self.amqp_cache,
                    &self.security_manager,
                )
                .await;
                if is_close {
                    basic::requeue_connection(connection_id, &self.basic_ctx()).await;
                }
                frame
            }
            AMQPClass::Channel(method) => {
                use amq_protocol::protocol::channel::AMQPMethod as ChanMethod;
                let is_close = matches!(method, ChanMethod::Close(_) | ChanMethod::CloseOk(_));
                let frame = channel::process_channel_full(
                    channel_id,
                    method,
                    connection_id,
                    &self.amqp_cache,
                );
                if is_close {
                    basic::requeue_channel(connection_id, channel_id, &self.basic_ctx()).await;
                }
                frame
            }
            AMQPClass::Exchange(method) => {
                exchange::process_exchange_full(
                    channel_id,
                    method,
                    connection_id,
                    &self.amqp_cache,
                    &self.storage_driver_manager,
                )
                .await
            }
            AMQPClass::Queue(method) => {
                queue::process_queue_full(
                    channel_id,
                    method,
                    connection_id,
                    &self.amqp_cache,
                    &self.storage_driver_manager,
                )
                .await
            }
            AMQPClass::Basic(method) => {
                basic::process_basic_full(channel_id, method, connection_id, &self.basic_ctx())
                    .await
            }
            AMQPClass::Tx(method) => tx::process_tx(channel_id, method),
            // access.request is deprecated and permissions are configured on the
            // broker side, but real RabbitMQ still acks it unconditionally rather
            // than leaving the client hanging, so we match that behavior.
            AMQPClass::Access(amq_protocol::protocol::access::AMQPMethod::Request(_)) => {
                Some(AMQPFrame::Method(
                    channel_id,
                    AMQPClass::Access(amq_protocol::protocol::access::AMQPMethod::RequestOk(
                        amq_protocol::protocol::access::RequestOk {},
                    )),
                ))
            }
            AMQPClass::Access(_) => None,
            AMQPClass::Confirm(method) => basic::process_confirm(channel_id, method),
        };
        if result.is_none() {
            use amq_protocol::protocol::basic::AMQPMethod as B;
            let is_no_reply = matches!(
                class,
                AMQPClass::Basic(
                    B::Ack(_)
                        | B::Nack(_)
                        | B::Reject(_)
                        | B::Publish(_)
                        | B::RecoverAsync(_)
                        | B::Get(_)
                ) | AMQPClass::Connection(
                    amq_protocol::protocol::connection::AMQPMethod::TuneOk(_)
                        | amq_protocol::protocol::connection::AMQPMethod::CloseOk(_)
                ) | AMQPClass::Channel(amq_protocol::protocol::channel::AMQPMethod::CloseOk(_))
            );
            if !is_no_reply {
                warn!(
                    "AMQP method not yet implemented: channel={} class={:?}",
                    channel_id, class
                );
            }
        }
        result
    }
}
