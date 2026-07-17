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
use amq_protocol::protocol::queue::{AMQPMethod as QueueMethod, DeclareOk as QueueDeclareOk};
use amq_protocol::protocol::AMQPClass;
use async_trait::async_trait;
use common_base::error::common::CommonError;
use common_base::uuid::unique_id;
use dashmap::DashMap;
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::amqp::binding::{AmqpBinding, AmqpBindingDestinationType};
use metadata_struct::amqp::exchange::{AmqpExchange, AmqpExchangeType};
use metadata_struct::amqp::queue::AmqpQueue;
use metadata_struct::connection::NetworkConnection;
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
use crate::core::cache::AmqpCacheManager;
use crate::storage::binding::BindingStorage;
use crate::storage::exchange::ExchangeStorage;
use crate::storage::queue::QueueStorage;

pub fn create_command() -> ArcCommandAdapter {
    Arc::new(Box::new(AmqpHandlerCommand::new_stateless()))
}

pub fn create_command_with_state(
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    amqp_cache: Arc<AmqpCacheManager>,
) -> ArcCommandAdapter {
    Arc::new(Box::new(AmqpHandlerCommand::new(
        connection_manager,
        storage_driver_manager,
        amqp_cache,
    )))
}

/// A Basic.Publish method frame followed by a not-yet-complete Content
/// Header/Body sequence, keyed by (connection_id, channel_id) until the full
/// body has arrived and the message can be written to storage.
struct PendingPublish {
    routing_key: String,
    exchange: String,
    body_size: Option<u64>,
    body: Vec<u8>,
}

#[derive(Clone)]
pub struct AmqpHandlerCommand {
    connection_manager: Option<Arc<ConnectionManager>>,
    storage_driver_manager: Option<Arc<StorageDriverManager>>,
    amqp_cache: Option<Arc<AmqpCacheManager>>,
    // (connection_id, queue_name) -> per-shard offsets
    shard_offsets: Arc<DashMap<(u64, String), HashMap<String, u64>>>,
    // (connection_id, channel_id) -> in-flight publish awaiting its content frames
    pending_publish: Arc<DashMap<(u64, u16), PendingPublish>>,
}

impl AmqpHandlerCommand {
    pub fn new_stateless() -> Self {
        AmqpHandlerCommand {
            connection_manager: None,
            storage_driver_manager: None,
            amqp_cache: None,
            shard_offsets: Arc::new(DashMap::new()),
            pending_publish: Arc::new(DashMap::new()),
        }
    }

    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        amqp_cache: Arc<AmqpCacheManager>,
    ) -> Self {
        AmqpHandlerCommand {
            connection_manager: Some(connection_manager),
            storage_driver_manager: Some(storage_driver_manager),
            amqp_cache: Some(amqp_cache),
            shard_offsets: Arc::new(DashMap::new()),
            pending_publish: Arc::new(DashMap::new()),
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
                self.process_content_header(connection_id, *channel_id, *class_id, header)
                    .await
            }
            AMQPFrame::Body(channel_id, data) => {
                self.process_content_body(connection_id, *channel_id, data)
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
            AMQPClass::Connection(method) => connection::process_connection(channel_id, method),
            AMQPClass::Channel(method) => channel::process_channel(channel_id, method),
            AMQPClass::Exchange(method) => self.process_exchange(channel_id, method).await,
            AMQPClass::Queue(method) => self.process_queue(channel_id, method).await,
            AMQPClass::Basic(method) => self.process_basic(channel_id, method, connection_id).await,
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
            AMQPMethod::Publish(publish) => {
                self.pending_publish.insert(
                    (connection_id, channel_id),
                    PendingPublish {
                        routing_key: publish.routing_key.to_string(),
                        exchange: publish.exchange.to_string(),
                        body_size: None,
                        body: Vec::new(),
                    },
                );
                None
            }
            other => basic::process_basic(channel_id, other),
        }
    }

    async fn process_queue(
        &self,
        channel_id: u16,
        method: &amq_protocol::protocol::queue::AMQPMethod,
    ) -> Option<AMQPFrame> {
        match method {
            QueueMethod::Declare(declare) => self.process_queue_declare(channel_id, declare).await,
            QueueMethod::Delete(delete) => self.process_queue_delete(channel_id, delete).await,
            QueueMethod::Bind(bind) => self.process_queue_bind(channel_id, bind).await,
            QueueMethod::Unbind(unbind) => self.process_queue_unbind(channel_id, unbind).await,
            other => queue::process_queue(channel_id, other),
        }
    }

    async fn process_queue_declare(
        &self,
        channel_id: u16,
        declare: &amq_protocol::protocol::queue::Declare,
    ) -> Option<AMQPFrame> {
        let queue_name = if declare.queue.as_str().is_empty() {
            format!("amqp-{}", unique_id())
        } else {
            declare.queue.to_string()
        };

        if let Some(sdm) = &self.storage_driver_manager {
            // The physical message shard: needed whether or not the queue's own
            // declare metadata is durable — something has to hold its messages
            // while it's alive.
            if queue::declare_amqp_queue(sdm, &queue_name).await.is_none() {
                warn!("AMQP Queue.Declare failed for queue={}", queue_name);
            }

            let arguments: HashMap<String, String> = declare
                .arguments
                .inner()
                .iter()
                .map(|(k, v)| (k.as_str().to_string(), format!("{:?}", v)))
                .collect();
            let amqp_queue = AmqpQueue::new(
                DEFAULT_TENANT,
                &queue_name,
                declare.durable,
                declare.exclusive,
                declare.auto_delete,
                arguments,
            );
            let storage = QueueStorage::new(sdm.engine_storage_handler.client_pool.clone());
            match storage.set_queue(&amqp_queue).await {
                Ok(()) => {
                    if let Some(cache) = &self.amqp_cache {
                        cache.set_queue(amqp_queue);
                    }
                }
                Err(e) => warn!(
                    "AMQP Queue.Declare metadata write failed for {}: {}",
                    queue_name, e
                ),
            }
        }

        Some(AMQPFrame::Method(
            channel_id,
            AMQPClass::Queue(QueueMethod::DeclareOk(QueueDeclareOk {
                queue: queue_name.into(),
                message_count: 0,
                consumer_count: 0,
            })),
        ))
    }

    async fn process_queue_delete(
        &self,
        channel_id: u16,
        delete: &amq_protocol::protocol::queue::Delete,
    ) -> Option<AMQPFrame> {
        use amq_protocol::protocol::queue::DeleteOk;

        let queue_name = delete.queue.to_string();
        if let Some(sdm) = &self.storage_driver_manager {
            let storage = QueueStorage::new(sdm.engine_storage_handler.client_pool.clone());
            match storage.delete_queue(DEFAULT_TENANT, &queue_name).await {
                Ok(()) => {
                    if let Some(cache) = &self.amqp_cache {
                        cache.remove_queue(DEFAULT_TENANT, &queue_name);
                    }
                }
                Err(e) => warn!("AMQP Queue.Delete failed for {}: {}", queue_name, e),
            }
        }

        Some(AMQPFrame::Method(
            channel_id,
            AMQPClass::Queue(QueueMethod::DeleteOk(DeleteOk { message_count: 0 })),
        ))
    }

    async fn process_queue_bind(
        &self,
        channel_id: u16,
        bind: &amq_protocol::protocol::queue::Bind,
    ) -> Option<AMQPFrame> {
        use amq_protocol::protocol::queue::BindOk;

        if let Some(sdm) = &self.storage_driver_manager {
            let arguments: HashMap<String, String> = bind
                .arguments
                .inner()
                .iter()
                .map(|(k, v)| (k.as_str().to_string(), format!("{:?}", v)))
                .collect();
            let binding = AmqpBinding::new(
                DEFAULT_TENANT,
                bind.exchange.as_str(),
                bind.queue.as_str(),
                AmqpBindingDestinationType::Queue,
                bind.routing_key.as_str(),
                arguments,
            );
            let storage = BindingStorage::new(sdm.engine_storage_handler.client_pool.clone());
            match storage.set_binding(&binding).await {
                Ok(()) => {
                    if let Some(cache) = &self.amqp_cache {
                        cache.set_binding(binding);
                    }
                }
                Err(e) => warn!("AMQP Queue.Bind failed: {}", e),
            }
        }

        Some(AMQPFrame::Method(
            channel_id,
            AMQPClass::Queue(QueueMethod::BindOk(BindOk {})),
        ))
    }

    async fn process_queue_unbind(
        &self,
        channel_id: u16,
        unbind: &amq_protocol::protocol::queue::Unbind,
    ) -> Option<AMQPFrame> {
        use amq_protocol::protocol::queue::UnbindOk;

        if let Some(sdm) = &self.storage_driver_manager {
            let storage = BindingStorage::new(sdm.engine_storage_handler.client_pool.clone());
            let destination_type = AmqpBindingDestinationType::Queue;
            match storage
                .delete_binding(
                    DEFAULT_TENANT,
                    unbind.exchange.as_str(),
                    unbind.queue.as_str(),
                    &destination_type,
                    unbind.routing_key.as_str(),
                )
                .await
            {
                Ok(()) => {
                    if let Some(cache) = &self.amqp_cache {
                        let key = format!(
                            "{}/{}/{}/{}",
                            unbind.exchange.as_str(),
                            destination_type.as_str(),
                            unbind.queue.as_str(),
                            unbind.routing_key.as_str()
                        );
                        cache.remove_binding(DEFAULT_TENANT, &key);
                    }
                }
                Err(e) => warn!("AMQP Queue.Unbind failed: {}", e),
            }
        }

        Some(AMQPFrame::Method(
            channel_id,
            AMQPClass::Queue(QueueMethod::UnbindOk(UnbindOk {})),
        ))
    }

    async fn process_exchange(
        &self,
        channel_id: u16,
        method: &amq_protocol::protocol::exchange::AMQPMethod,
    ) -> Option<AMQPFrame> {
        use amq_protocol::protocol::exchange::AMQPMethod;
        match method {
            AMQPMethod::Declare(declare) => {
                self.process_exchange_declare(channel_id, declare).await
            }
            AMQPMethod::Delete(delete) => self.process_exchange_delete(channel_id, delete).await,
            other => exchange::process_exchange(channel_id, other),
        }
    }

    async fn process_exchange_declare(
        &self,
        channel_id: u16,
        declare: &amq_protocol::protocol::exchange::Declare,
    ) -> Option<AMQPFrame> {
        use amq_protocol::protocol::exchange::{AMQPMethod as ExchangeMethod, DeclareOk};

        let exchange_name = declare.exchange.to_string();
        let exchange_type = AmqpExchangeType::from_str_opt(declare.kind.as_str()).unwrap_or_else(|| {
            warn!(
                "AMQP Exchange.Declare: unrecognized exchange type '{}' for {}, defaulting to direct",
                declare.kind.as_str(),
                exchange_name
            );
            AmqpExchangeType::Direct
        });
        let arguments: HashMap<String, String> = declare
            .arguments
            .inner()
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), format!("{:?}", v)))
            .collect();

        if let Some(sdm) = &self.storage_driver_manager {
            let exchange = AmqpExchange::new(
                DEFAULT_TENANT,
                &exchange_name,
                exchange_type,
                declare.durable,
                declare.auto_delete,
                declare.internal,
                arguments,
            );
            let storage = ExchangeStorage::new(sdm.engine_storage_handler.client_pool.clone());
            match storage.set_exchange(&exchange).await {
                Ok(()) => {
                    if let Some(cache) = &self.amqp_cache {
                        cache.set_exchange(exchange);
                    }
                }
                Err(e) => warn!("AMQP Exchange.Declare failed for {}: {}", exchange_name, e),
            }
        }

        Some(AMQPFrame::Method(
            channel_id,
            AMQPClass::Exchange(ExchangeMethod::DeclareOk(DeclareOk {})),
        ))
    }

    async fn process_exchange_delete(
        &self,
        channel_id: u16,
        delete: &amq_protocol::protocol::exchange::Delete,
    ) -> Option<AMQPFrame> {
        use amq_protocol::protocol::exchange::{AMQPMethod as ExchangeMethod, DeleteOk};

        let exchange_name = delete.exchange.to_string();
        if let Some(sdm) = &self.storage_driver_manager {
            let storage = ExchangeStorage::new(sdm.engine_storage_handler.client_pool.clone());
            match storage
                .delete_exchange(DEFAULT_TENANT, &exchange_name)
                .await
            {
                Ok(()) => {
                    if let Some(cache) = &self.amqp_cache {
                        cache.remove_exchange(DEFAULT_TENANT, &exchange_name);
                    }
                }
                Err(e) => warn!("AMQP Exchange.Delete failed for {}: {}", exchange_name, e),
            }
        }

        Some(AMQPFrame::Method(
            channel_id,
            AMQPClass::Exchange(ExchangeMethod::DeleteOk(DeleteOk {})),
        ))
    }

    /// Content Header frame: carries body_size for the Basic.Publish that preceded
    /// it. A zero-length body means the message is already complete.
    async fn process_content_header(
        &self,
        connection_id: u64,
        channel_id: u16,
        class_id: u16,
        header: &AMQPContentHeader,
    ) -> Option<AMQPFrame> {
        if class_id != 60 {
            // Only the Basic class (60) carries publishable message content.
            return None;
        }
        let key = (connection_id, channel_id);
        let complete = match self.pending_publish.get_mut(&key) {
            Some(mut entry) => {
                entry.body_size = Some(header.body_size);
                header.body_size == 0
            }
            None => false,
        };
        if complete {
            if let Some((_, pending)) = self.pending_publish.remove(&key) {
                self.finalize_publish(pending).await;
            }
        }
        None
    }

    /// Content Body frame: one chunk of the message payload. A message may be
    /// split across multiple Body frames up to the negotiated frame_max.
    async fn process_content_body(
        &self,
        connection_id: u64,
        channel_id: u16,
        data: &[u8],
    ) -> Option<AMQPFrame> {
        let key = (connection_id, channel_id);
        let complete = match self.pending_publish.get_mut(&key) {
            Some(mut entry) => {
                entry.body.extend_from_slice(data);
                matches!(entry.body_size, Some(size) if entry.body.len() as u64 >= size)
            }
            None => false,
        };
        if complete {
            if let Some((_, pending)) = self.pending_publish.remove(&key) {
                self.finalize_publish(pending).await;
            }
        }
        None
    }

    /// Writes a fully-assembled AMQP message to storage. The default exchange
    /// ("") routes directly by routing_key == queue name; named exchanges are not
    /// yet modeled, so routing_key is always treated as the target queue name.
    async fn finalize_publish(&self, pending: PendingPublish) {
        let Some(sdm) = self.storage_driver_manager.clone() else {
            return;
        };
        if pending.routing_key.is_empty() {
            warn!(
                "AMQP Basic.Publish with empty routing key ignored (exchange={})",
                pending.exchange
            );
            return;
        }

        let topic_name = pending.routing_key;
        let record = AdapterWriteRecord::new(topic_name.clone(), pending.body);
        match sdm
            .write(
                DEFAULT_TENANT,
                &topic_name,
                std::slice::from_ref(&record),
                1,
            )
            .await
        {
            Ok(_) => {}
            Err(CommonError::TopicNotFoundInBrokerCache(_, _)) => {
                // Published to a queue that was never explicitly declared (common
                // with the default exchange): declare it on the fly, then retry.
                if queue::declare_amqp_queue(&sdm, &topic_name).await.is_some() {
                    if let Err(e) = sdm.write(DEFAULT_TENANT, &topic_name, &[record], 1).await {
                        error!(
                            "AMQP Basic.Publish retry write failed for {}: {}",
                            topic_name, e
                        );
                    }
                } else {
                    error!(
                        "AMQP Basic.Publish dropped: queue {} does not exist and could not be created",
                        topic_name
                    );
                }
            }
            Err(e) => error!("AMQP Basic.Publish write failed for {}: {}", topic_name, e),
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

                            let body = record.data.to_vec();
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

                let body = record.data.to_vec();
                let body_size = body.len() as u64;

                // AMQP requires the Method frame before its Content Header/Body, so
                // GetOk must be written first, not returned to be sent afterward.
                let get_ok_frame = AMQPFrame::Method(
                    channel_id,
                    AMQPClass::Basic(BasicMethod::GetOk(GetOk {
                        delivery_tag: record.metadata.offset + 1,
                        redelivered: false,
                        exchange: "".into(),
                        routing_key: queue.into(),
                        message_count: 0,
                    })),
                );
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

                for frame in [get_ok_frame, header_frame, body_frame] {
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

                None
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
