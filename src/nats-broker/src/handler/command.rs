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

use crate::core::cache::NatsCacheManager;
use crate::nats::{connect, ping, publish, subscribe};
use crate::push::manager::NatsSubscribeManager;
use async_trait::async_trait;
use common_security::manager::SecurityManager;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnection;
use network_server::command::Command;
use network_server::common::connection_manager::ConnectionManager;
use network_server::common::packet::ResponsePackage;
use protocol::nats::packet::NatsPacket;
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

#[derive(Clone)]
pub struct NatsProcessContext {
    pub connect_id: u64,
    pub connection_manager: Arc<ConnectionManager>,
    pub cache_manager: Arc<NatsCacheManager>,
    pub subscribe_manager: Arc<NatsSubscribeManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub client_pool: Arc<ClientPool>,
    pub security_manager: Arc<SecurityManager>,
}

#[derive(Clone)]
pub struct NatsHandlerCommand {
    pub connection_manager: Arc<ConnectionManager>,
    pub cache_manager: Arc<NatsCacheManager>,
    pub subscribe_manager: Arc<NatsSubscribeManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub client_pool: Arc<ClientPool>,
    pub security_manager: Arc<SecurityManager>,
}

impl NatsHandlerCommand {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<NatsCacheManager>,
        subscribe_manager: Arc<NatsSubscribeManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        client_pool: Arc<ClientPool>,
        security_manager: Arc<SecurityManager>,
    ) -> Self {
        NatsHandlerCommand {
            connection_manager,
            cache_manager,
            subscribe_manager,
            storage_driver_manager,
            client_pool,
            security_manager,
        }
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

        // Refresh heartbeat on every inbound client message.
        self.connection_manager
            .report_heartbeat(connection_id, common_base::tools::now_second());

        let ctx = NatsProcessContext {
            connect_id: connection_id,
            connection_manager: self.connection_manager.clone(),
            cache_manager: self.cache_manager.clone(),
            subscribe_manager: self.subscribe_manager.clone(),
            storage_driver_manager: self.storage_driver_manager.clone(),
            client_pool: self.client_pool.clone(),
            security_manager: self.security_manager.clone(),
        };

        // Helper: convert Result<(), NatsPacket> into Option<NatsPacket> with verbose.
        let verbose = self
            .cache_manager
            .get_connection(connection_id)
            .map(|c| c.verbose)
            .unwrap_or(false);

        let resp_packet: Option<NatsPacket> = match &packet {
            NatsPacket::Connect(req) => {
                // verbose is not yet in cache when CONNECT arrives; read from req directly
                let verbose = req.verbose;
                match connect::process_connect(&ctx, req) {
                    Ok(()) => verbose.then_some(NatsPacket::Ok),
                    Err(e) => Some(e),
                }
            }

            NatsPacket::Pub {
                subject,
                reply_to,
                payload,
            } => {
                match publish::process_pub(&ctx, subject, reply_to.as_deref(), &None, payload).await
                {
                    Ok(Some(pkt)) => Some(pkt), // server-initiated (e.g. mq9 reply)
                    Ok(None) => verbose.then_some(NatsPacket::Ok),
                    Err(e) => Some(e),
                }
            }

            NatsPacket::HPub {
                subject,
                reply_to,
                headers,
                payload,
            } => {
                match publish::process_pub(
                    &ctx,
                    subject,
                    reply_to.as_deref(),
                    &Some(headers.clone()),
                    payload,
                )
                .await
                {
                    Ok(Some(pkt)) => Some(pkt),
                    Ok(None) => verbose.then_some(NatsPacket::Ok),
                    Err(e) => Some(e),
                }
            }

            NatsPacket::Sub {
                subject,
                queue_group,
                sid,
            } => match subscribe::process_sub(&ctx, subject, queue_group.as_deref(), sid).await {
                Ok(()) => verbose.then_some(NatsPacket::Ok),
                Err(e) => Some(e),
            },

            NatsPacket::Unsub { sid, max_msgs } => {
                match subscribe::process_unsub(&ctx, sid, *max_msgs).await {
                    Ok(()) => verbose.then_some(NatsPacket::Ok),
                    Err(e) => Some(e),
                }
            }

            NatsPacket::Ping => ping::process_ping(connection_id, &self.connection_manager),

            NatsPacket::Pong => ping::process_pong(connection_id, &self.connection_manager),

            // Server-to-client packets; not expected from a client
            NatsPacket::Info(_)
            | NatsPacket::Msg { .. }
            | NatsPacket::HMsg { .. }
            | NatsPacket::Ok
            | NatsPacket::Err(_) => None,
        };

        Some(ResponsePackage::new(
            connection_id,
            RobustMQPacket::NATS(resp_packet?),
        ))
    }
}

pub fn create_command(
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<NatsCacheManager>,
    subscribe_manager: Arc<NatsSubscribeManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    client_pool: Arc<ClientPool>,
    security_manager: Arc<SecurityManager>,
) -> Arc<Box<dyn Command + Send + Sync>> {
    Arc::new(Box::new(NatsHandlerCommand::new(
        connection_manager,
        cache_manager,
        subscribe_manager,
        storage_driver_manager,
        client_pool,
        security_manager,
    )))
}
