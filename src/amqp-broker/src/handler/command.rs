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

use std::collections::HashMap;
use std::sync::Arc;

use amq_protocol::frame::{AMQPContentHeader, AMQPFrame};
use amq_protocol::protocol::basic::AMQPMethod as BasicMethod;
use amq_protocol::protocol::basic::{AMQPProperties, ConsumeOk, Deliver, GetEmpty, GetOk};
use amq_protocol::protocol::AMQPClass;
use async_trait::async_trait;
use dashmap::DashMap;
use metadata_struct::connection::NetworkConnection;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use metadata_struct::tenant::DEFAULT_TENANT;
use network_server::command::{ArcCommandAdapter, Command};
use network_server::common::connection_manager::ConnectionManager;
use network_server::common::packet::ResponsePackage;
use protocol::robust::{
    AmqpWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use std::net::SocketAddr;
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::time::sleep;
use tracing::{debug, error, warn};

use crate::amqp::{basic, channel, connection, exchange, queue, tx};

pub fn create_command() -> ArcCommandAdapter {
    Arc::new(Box::new(AmqpHandlerCommand::new_stateless()))
}

pub fn create_command_with_state(
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
) -> ArcCommandAdapter {
    Arc::new(Box::new(AmqpHandlerCommand::new(
        connection_manager,
        storage_driver_manager,
    )))
}

#[derive(Clone)]
pub struct AmqpHandlerCommand {
    connection_manager: Option<Arc<ConnectionManager>>,
    storage_driver_manager: Option<Arc<StorageDriverManager>>,
    // (connection_id, queue_name) -> per-shard offsets
    shard_offsets: Arc<DashMap<(u64, String), HashMap<String, u64>>>,
}

impl AmqpHandlerCommand {
    pub fn new_stateless() -> Self {
        AmqpHandlerCommand {
            connection_manager: None,
            storage_driver_manager: None,
            shard_offsets: Arc::new(DashMap::new()),
        }
    }

    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
    ) -> Self {
        AmqpHandlerCommand {
            connection_manager: Some(connection_manager),
            storage_driver_manager: Some(storage_driver_manager),
            shard_offsets: Arc::new(DashMap::new()),
        }
    }
}

impl Default for AmqpHandlerCommand {
    fn default() -> Self {
        Self::new_stateless()
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
            AMQPFrame::ProtocolHeader(_) => connection::process_protocol_header(),
            AMQPFrame::Heartbeat(channel_id) => connection::process_heartbeat(*channel_id),
            AMQPFrame::Header(channel_id, class_id, header) => {
                basic::process_header(*channel_id, *class_id, header)
            }
            AMQPFrame::Body(channel_id, data) => basic::process_body(*channel_id, data),
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
            AMQPClass::Connection(method) => connection::process_connection(channel_id, method),
            AMQPClass::Channel(method) => channel::process_channel(channel_id, method),
            AMQPClass::Exchange(method) => exchange::process_exchange(channel_id, method),
            AMQPClass::Queue(method) => queue::process_queue(channel_id, method),
            AMQPClass::Basic(method) => self.process_basic(channel_id, method, connection_id).await,
            AMQPClass::Tx(method) => tx::process_tx(channel_id, method),
            AMQPClass::Access(_) => None,
            AMQPClass::Confirm(method) => basic::process_confirm(channel_id, method),
        };
        if result.is_none() {
            use amq_protocol::protocol::basic::AMQPMethod as B;
            let is_no_reply = matches!(
                class,
                AMQPClass::Basic(
                    B::Ack(_) | B::Nack(_) | B::Reject(_) | B::Publish(_) | B::RecoverAsync(_)
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

    async fn process_basic(
        &self,
        channel_id: u16,
        method: &amq_protocol::protocol::basic::AMQPMethod,
        connection_id: u64,
    ) -> Option<AMQPFrame> {
        use amq_protocol::protocol::basic::AMQPMethod;
        match method {
            AMQPMethod::Get(get) => {
                self.process_get(channel_id, get.queue.as_str(), connection_id)
                    .await
            }
            AMQPMethod::Consume(consume) => {
                self.process_consume(
                    channel_id,
                    consume.queue.as_str(),
                    consume.consumer_tag.as_str(),
                    connection_id,
                )
                .await
            }
            other => basic::process_basic(channel_id, other),
        }
    }

    async fn process_consume(
        &self,
        channel_id: u16,
        queue: &str,
        consumer_tag: &str,
        connection_id: u64,
    ) -> Option<AMQPFrame> {
        let (cm, sdm) = match (&self.connection_manager, &self.storage_driver_manager) {
            (Some(cm), Some(sdm)) => (cm.clone(), sdm.clone()),
            _ => {
                warn!("AMQP Basic.Consume: storage not configured");
                return Some(AMQPFrame::Method(
                    channel_id,
                    AMQPClass::Basic(BasicMethod::ConsumeOk(ConsumeOk {
                        consumer_tag: consumer_tag.into(),
                    })),
                ));
            }
        };

        let queue = queue.to_string();
        let consumer_tag = consumer_tag.to_string();
        let consumer_tag_resp = consumer_tag.clone();
        let read_config = AdapterReadConfig::new();

        tokio::spawn(async move {
            // key: shard_name -> next offset to read
            let mut shard_offsets: HashMap<String, u64> = HashMap::new();
            let mut delivery_tag: u64 = 1;

            loop {
                match sdm
                    .read_by_offset(DEFAULT_TENANT, &queue, &shard_offsets, &read_config)
                    .await
                {
                    Ok(records) if records.is_empty() => {
                        sleep(Duration::from_millis(100)).await;
                    }
                    Ok(records) => {
                        for record in &records {
                            shard_offsets
                                .insert(record.metadata.shard.clone(), record.metadata.offset + 1);

                            let body = match MqttMessage::decode(&record.data) {
                                Ok(msg) => msg.payload.to_vec(),
                                Err(_) => record.data.to_vec(),
                            };
                            let body_size = body.len() as u64;

                            // Deliver method frame
                            let deliver_frame = AMQPFrame::Method(
                                channel_id,
                                AMQPClass::Basic(BasicMethod::Deliver(Deliver {
                                    consumer_tag: consumer_tag.clone().into(),
                                    delivery_tag,
                                    redelivered: false,
                                    exchange: "".into(),
                                    routing_key: queue.clone().into(),
                                })),
                            );
                            // Content header frame
                            let header_frame = AMQPFrame::Header(
                                channel_id,
                                60,
                                Box::new(AMQPContentHeader {
                                    class_id: 60,
                                    body_size,
                                    properties: AMQPProperties::default(),
                                }),
                            );
                            // Body frame
                            let body_frame = AMQPFrame::Body(channel_id, body);

                            for frame in [deliver_frame, header_frame, body_frame] {
                                let wrapper = RobustMQPacketWrapper {
                                    protocol: RobustMQProtocol::AMQP,
                                    extend: RobustMQWrapperExtend::AMQP(AmqpWrapperExtend {}),
                                    packet: RobustMQPacket::AMQP(frame),
                                };
                                if let Err(e) = cm.write_tcp_frame(connection_id, wrapper).await {
                                    error!(connection_id, "AMQP Deliver write failed: {}", e);
                                    return;
                                }
                            }

                            delivery_tag += 1;
                        }
                    }
                    Err(e) => {
                        error!("AMQP Basic.Consume storage read error on {}: {}", queue, e);
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });

        // Respond ConsumeOk immediately
        Some(AMQPFrame::Method(
            channel_id,
            AMQPClass::Basic(BasicMethod::ConsumeOk(ConsumeOk {
                consumer_tag: consumer_tag_resp.into(),
            })),
        ))
    }

    async fn process_get(
        &self,
        channel_id: u16,
        queue: &str,
        connection_id: u64,
    ) -> Option<AMQPFrame> {
        let (cm, sdm) = match (&self.connection_manager, &self.storage_driver_manager) {
            (Some(cm), Some(sdm)) => (cm, sdm),
            _ => {
                warn!("AMQP Basic.Get: storage not configured");
                return Some(AMQPFrame::Method(
                    channel_id,
                    AMQPClass::Basic(BasicMethod::GetEmpty(GetEmpty {})),
                ));
            }
        };

        let key = (connection_id, queue.to_string());
        let mut offsets = self
            .shard_offsets
            .get(&key)
            .map(|r| r.clone())
            .unwrap_or_default();

        let read_config = AdapterReadConfig::new();
        match sdm
            .read_by_offset(DEFAULT_TENANT, queue, &offsets, &read_config)
            .await
        {
            Ok(records) if records.is_empty() => {
                // No message available
                Some(AMQPFrame::Method(
                    channel_id,
                    AMQPClass::Basic(BasicMethod::GetEmpty(GetEmpty {})),
                ))
            }
            Ok(records) => {
                let record = &records[0];
                // Advance shard offset
                offsets.insert(record.metadata.shard.clone(), record.metadata.offset + 1);
                self.shard_offsets.insert(key, offsets);

                let body = match MqttMessage::decode(&record.data) {
                    Ok(msg) => msg.payload.to_vec(),
                    Err(_) => record.data.to_vec(),
                };
                let body_size = body.len() as u64;

                // Send Header and Body directly; return GetOk as the method frame
                let header_frame = AMQPFrame::Header(
                    channel_id,
                    60, // basic class_id
                    Box::new(AMQPContentHeader {
                        class_id: 60,
                        body_size,
                        properties: AMQPProperties::default(),
                    }),
                );
                let body_frame = AMQPFrame::Body(channel_id, body);

                // Write header and body directly to the connection
                for frame in [header_frame, body_frame] {
                    let wrapper = RobustMQPacketWrapper {
                        protocol: RobustMQProtocol::AMQP,
                        extend: RobustMQWrapperExtend::AMQP(AmqpWrapperExtend {}),
                        packet: RobustMQPacket::AMQP(frame),
                    };
                    if let Err(e) = cm.write_tcp_frame(connection_id, wrapper).await {
                        error!(connection_id, "AMQP Basic.Get write failed: {}", e);
                        return None;
                    }
                }

                Some(AMQPFrame::Method(
                    channel_id,
                    AMQPClass::Basic(BasicMethod::GetOk(GetOk {
                        delivery_tag: record.metadata.offset + 1,
                        redelivered: false,
                        exchange: "".into(),
                        routing_key: queue.into(),
                        message_count: 0,
                    })),
                ))
            }
            Err(e) => {
                error!("AMQP Basic.Get storage error for {}: {}", queue, e);
                Some(AMQPFrame::Method(
                    channel_id,
                    AMQPClass::Basic(BasicMethod::GetEmpty(GetEmpty {})),
                ))
            }
        }
    }
}
