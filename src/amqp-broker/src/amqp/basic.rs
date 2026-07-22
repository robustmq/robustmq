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
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use amq_protocol::frame::{AMQPContentHeader, AMQPFrame};
use amq_protocol::protocol::basic::{
    AMQPMethod, AMQPProperties, CancelOk, ConsumeOk, Deliver, GetEmpty, GetOk, QosOk, RecoverOk,
    Return,
};
use amq_protocol::protocol::confirm;
use amq_protocol::protocol::AMQPClass;
use amq_protocol::types::{AMQPValue, FieldTable};
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::record::{
    StorageRecord, StorageRecordProtocolData, StorageRecordProtocolDataAmqp,
};
use network_server::common::connection_manager::ConnectionManager;
use protocol::robust::{
    AmqpWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use storage_adapter::driver::StorageDriverManager;
use tokio::time::sleep;
use tracing::{debug, error, warn};

use crate::amqp::{offset, queue, route};
use crate::core::cache::{AmqpCacheManager, PendingPublish, UnackedEntry};
use crate::core::recovery::requeue_message;
use crate::core::unacked_index;

/// Maps the wire-level AMQPProperties from a Content Header frame onto the
/// shape stored alongside the message, so redelivery can reconstruct them
/// instead of always sending an empty property set.
fn properties_to_protocol_data(properties: &AMQPProperties) -> StorageRecordProtocolDataAmqp {
    StorageRecordProtocolDataAmqp {
        content_type: properties.content_type().as_ref().map(|s| s.to_string()),
        content_encoding: properties
            .content_encoding()
            .as_ref()
            .map(|s| s.to_string()),
        delivery_mode: *properties.delivery_mode(),
        priority: *properties.priority(),
        correlation_id: properties.correlation_id().as_ref().map(|s| s.to_string()),
        reply_to: properties.reply_to().as_ref().map(|s| s.to_string()),
        expiration: properties.expiration().as_ref().map(|s| s.to_string()),
        message_id: properties.message_id().as_ref().map(|s| s.to_string()),
        timestamp: *properties.timestamp(),
        kind: properties.kind().as_ref().map(|s| s.to_string()),
        user_id: properties.user_id().as_ref().map(|s| s.to_string()),
        app_id: properties.app_id().as_ref().map(|s| s.to_string()),
        cluster_id: properties.cluster_id().as_ref().map(|s| s.to_string()),
        headers: properties
            .headers()
            .as_ref()
            .map(|table| route::field_table_to_map(table).into_iter().collect())
            .unwrap_or_default(),
        redelivered: false,
    }
}

/// The inverse of `properties_to_protocol_data`: rebuilds the AMQPProperties
/// to send with a redelivered/fetched message from what was stored alongside it.
fn properties_from_record(record: &StorageRecord) -> AMQPProperties {
    match record
        .protocol_data
        .as_ref()
        .and_then(|pd| pd.amqp.as_ref())
    {
        Some(amqp) => properties_from_protocol_data(amqp),
        None => AMQPProperties::default(),
    }
}

fn properties_from_protocol_data(amqp: &StorageRecordProtocolDataAmqp) -> AMQPProperties {
    let mut properties = AMQPProperties::default();
    if let Some(v) = &amqp.content_type {
        properties = properties.with_content_type(v.as_str().into());
    }
    if let Some(v) = &amqp.content_encoding {
        properties = properties.with_content_encoding(v.as_str().into());
    }
    if !amqp.headers.is_empty() {
        let mut table = FieldTable::default();
        for (k, v) in &amqp.headers {
            table.insert(k.as_str().into(), AMQPValue::LongString(v.as_str().into()));
        }
        properties = properties.with_headers(table);
    }
    if let Some(v) = amqp.delivery_mode {
        properties = properties.with_delivery_mode(v);
    }
    if let Some(v) = amqp.priority {
        properties = properties.with_priority(v);
    }
    if let Some(v) = &amqp.correlation_id {
        properties = properties.with_correlation_id(v.as_str().into());
    }
    if let Some(v) = &amqp.reply_to {
        properties = properties.with_reply_to(v.as_str().into());
    }
    if let Some(v) = &amqp.expiration {
        properties = properties.with_expiration(v.as_str().into());
    }
    if let Some(v) = &amqp.message_id {
        properties = properties.with_message_id(v.as_str().into());
    }
    if let Some(v) = amqp.timestamp {
        properties = properties.with_timestamp(v);
    }
    if let Some(v) = &amqp.kind {
        properties = properties.with_type(v.as_str().into());
    }
    if let Some(v) = &amqp.user_id {
        properties = properties.with_user_id(v.as_str().into());
    }
    if let Some(v) = &amqp.app_id {
        properties = properties.with_app_id(v.as_str().into());
    }
    if let Some(v) = &amqp.cluster_id {
        properties = properties.with_cluster_id(v.as_str().into());
    }
    properties
}

pub(crate) struct BasicCtx {
    pub connection_manager: Arc<ConnectionManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub amqp_cache: Arc<AmqpCacheManager>,
}

pub(crate) async fn process_basic_full(
    channel_id: u16,
    method: &AMQPMethod,
    connection_id: u64,
    ctx: &BasicCtx,
) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Get(get) => {
            process_get(
                channel_id,
                get.queue.as_str(),
                get.no_ack,
                connection_id,
                ctx,
            )
            .await
        }
        AMQPMethod::Ack(ack) => {
            process_settle(
                Some(ack.delivery_tag),
                ack.multiple,
                false,
                connection_id,
                channel_id,
                ctx,
            )
            .await;
            None
        }
        AMQPMethod::Nack(nack) => {
            process_settle(
                Some(nack.delivery_tag),
                nack.multiple,
                nack.requeue,
                connection_id,
                channel_id,
                ctx,
            )
            .await;
            None
        }
        AMQPMethod::Reject(reject) => {
            process_settle(
                Some(reject.delivery_tag),
                false,
                reject.requeue,
                connection_id,
                channel_id,
                ctx,
            )
            .await;
            None
        }
        AMQPMethod::RecoverAsync(recover) => {
            process_settle(None, false, true, connection_id, channel_id, ctx).await;
            let _ = recover.requeue;
            None
        }
        AMQPMethod::Recover(recover) => {
            process_settle(None, false, true, connection_id, channel_id, ctx).await;
            let _ = recover.requeue;
            Some(AMQPFrame::Method(
                channel_id,
                AMQPClass::Basic(AMQPMethod::RecoverOk(RecoverOk {})),
            ))
        }
        AMQPMethod::Consume(consume) => {
            process_consume(
                channel_id,
                consume.queue.as_str(),
                consume.consumer_tag.as_str(),
                connection_id,
                ctx,
            )
            .await
        }
        AMQPMethod::Publish(publish) => {
            ctx.amqp_cache.pending_publish().insert(
                (connection_id, channel_id),
                PendingPublish {
                    tenant: ctx.amqp_cache.tenant_for(connection_id),
                    routing_key: publish.routing_key.to_string(),
                    exchange: publish.exchange.to_string(),
                    mandatory: publish.mandatory,
                    headers: HashMap::new(),
                    properties: StorageRecordProtocolDataAmqp::default(),
                    body_size: None,
                    body: Vec::new(),
                },
            );
            None
        }
        other => process_basic(channel_id, other),
    }
}

/// Content Header frame: carries body_size for the Basic.Publish that preceded
/// it. A zero-length body means the message is already complete.
pub(crate) async fn process_content_header_full(
    connection_id: u64,
    channel_id: u16,
    class_id: u16,
    header: &AMQPContentHeader,
    ctx: &BasicCtx,
) -> Option<AMQPFrame> {
    if class_id != 60 {
        // Only the Basic class (60) carries publishable message content.
        return None;
    }
    let key = (connection_id, channel_id);
    let complete = match ctx.amqp_cache.pending_publish().get_mut(&key) {
        Some(mut entry) => {
            entry.body_size = Some(header.body_size);
            if let Some(headers) = header.properties.headers() {
                entry.headers = route::field_table_to_map(headers);
            }
            entry.properties = properties_to_protocol_data(&header.properties);
            header.body_size == 0
        }
        None => false,
    };
    if complete {
        if let Some((_, pending)) = ctx.amqp_cache.pending_publish().remove(&key) {
            finalize_publish(connection_id, channel_id, pending, ctx).await;
        }
    }
    None
}

/// Content Body frame: one chunk of the message payload. A message may be
/// split across multiple Body frames up to the negotiated frame_max.
pub(crate) async fn process_content_body_full(
    connection_id: u64,
    channel_id: u16,
    data: &[u8],
    ctx: &BasicCtx,
) -> Option<AMQPFrame> {
    let key = (connection_id, channel_id);
    let complete = match ctx.amqp_cache.pending_publish().get_mut(&key) {
        Some(mut entry) => {
            entry.body.extend_from_slice(data);
            matches!(entry.body_size, Some(size) if entry.body.len() as u64 >= size)
        }
        None => false,
    };
    if complete {
        if let Some((_, pending)) = ctx.amqp_cache.pending_publish().remove(&key) {
            finalize_publish(connection_id, channel_id, pending, ctx).await;
        }
    }
    None
}

/// Writes a fully-assembled AMQP message to storage. The default exchange
/// ("") is an implicit direct binding from every queue to itself by name;
/// named exchanges are routed via `route::resolve_queues`, which follows
/// their type (direct/fanout/topic/headers) and bindings, including
/// exchange-to-exchange chains. Unroutable `mandatory` publishes are
/// returned to the publisher via Basic.Return.
async fn finalize_publish(
    connection_id: u64,
    channel_id: u16,
    pending: PendingPublish,
    ctx: &BasicCtx,
) {
    if pending.exchange.is_empty() && pending.routing_key.is_empty() {
        warn!("AMQP Basic.Publish with empty routing key ignored on the default exchange");
        return;
    }

    let queues = if pending.exchange.is_empty() {
        vec![pending.routing_key.clone()]
    } else {
        route::resolve_queues(
            &ctx.amqp_cache,
            &pending.tenant,
            &pending.exchange,
            &pending.routing_key,
            &pending.headers,
        )
    };

    if queues.is_empty() {
        if pending.mandatory {
            send_basic_return(connection_id, channel_id, &pending, ctx).await;
        } else {
            debug!(
                "AMQP Basic.Publish unroutable (exchange={}, routing_key={}), dropped",
                pending.exchange, pending.routing_key
            );
        }
        return;
    }

    for queue_name in &queues {
        write_to_queue(
            &ctx.storage_driver_manager,
            &pending.tenant,
            queue_name,
            pending.body.clone(),
            &pending.properties,
        )
        .await;
    }
}

async fn write_to_queue(
    sdm: &Arc<StorageDriverManager>,
    tenant: &str,
    queue_name: &str,
    body: Vec<u8>,
    properties: &StorageRecordProtocolDataAmqp,
) {
    let record = AdapterWriteRecord::new(queue_name.to_string(), body).with_protocol_data(Some(
        StorageRecordProtocolData {
            amqp: Some(properties.clone()),
            ..Default::default()
        },
    ));
    match sdm
        .write(tenant, queue_name, std::slice::from_ref(&record), 1)
        .await
    {
        Ok(_) => {}
        Err(CommonError::TopicNotFoundInBrokerCache(_, _)) => {
            // Published to a queue that was never explicitly declared (common
            // with the default exchange): declare it on the fly, then retry.
            if queue::declare_amqp_queue(sdm, tenant, queue_name)
                .await
                .is_some()
            {
                if let Err(e) = sdm.write(tenant, queue_name, &[record], 1).await {
                    error!(
                        "AMQP Basic.Publish retry write failed for {}: {}",
                        queue_name, e
                    );
                }
            } else {
                error!(
                    "AMQP Basic.Publish dropped: queue {} does not exist and could not be created",
                    queue_name
                );
            }
        }
        Err(e) => error!("AMQP Basic.Publish write failed for {}: {}", queue_name, e),
    }
}

/// Sends an unroutable `mandatory` publish back to its publisher, per spec:
/// Basic.Return followed by the message's own content header and body.
async fn send_basic_return(
    connection_id: u64,
    channel_id: u16,
    pending: &PendingPublish,
    ctx: &BasicCtx,
) {
    let return_frame = AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::Return(Return {
            reply_code: 312,
            reply_text: "NO_ROUTE".into(),
            exchange: pending.exchange.clone().into(),
            routing_key: pending.routing_key.clone().into(),
        })),
    );
    let header_frame = AMQPFrame::Header(
        channel_id,
        60,
        Box::new(AMQPContentHeader {
            class_id: 60,
            body_size: pending.body.len() as u64,
            properties: properties_from_protocol_data(&pending.properties),
        }),
    );
    let body_frame = AMQPFrame::Body(channel_id, pending.body.clone());

    for frame in [return_frame, header_frame, body_frame] {
        let wrapper = RobustMQPacketWrapper {
            protocol: RobustMQProtocol::AMQP,
            extend: RobustMQWrapperExtend::AMQP(AmqpWrapperExtend {}),
            packet: RobustMQPacket::AMQP(frame),
        };
        if let Err(e) = ctx
            .connection_manager
            .write_tcp_frame(connection_id, wrapper)
            .await
        {
            error!(connection_id, "AMQP Basic.Return write failed: {}", e);
            return;
        }
    }
}

async fn process_consume(
    channel_id: u16,
    queue: &str,
    consumer_tag: &str,
    connection_id: u64,
    ctx: &BasicCtx,
) -> Option<AMQPFrame> {
    let cm = ctx.connection_manager.clone();
    let sdm = ctx.storage_driver_manager.clone();

    let tenant = ctx.amqp_cache.tenant_for(connection_id);
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
                .read_by_offset(&tenant, &queue, &shard_offsets, &read_config)
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
                            AMQPClass::Basic(AMQPMethod::Deliver(Deliver {
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
                                properties: properties_from_record(record),
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
        AMQPClass::Basic(AMQPMethod::ConsumeOk(ConsumeOk {
            consumer_tag: consumer_tag_resp.into(),
        })),
    ))
}

const MAX_CLAIM_ATTEMPTS: u32 = 5;

fn get_empty(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::GetEmpty(GetEmpty {})),
    ))
}

async fn process_get(
    channel_id: u16,
    queue: &str,
    no_ack: bool,
    connection_id: u64,
    ctx: &BasicCtx,
) -> Option<AMQPFrame> {
    let tenant = ctx.amqp_cache.tenant_for(connection_id);

    let Some(topic) = queue::declare_amqp_queue(&ctx.storage_driver_manager, &tenant, queue).await
    else {
        error!("AMQP Basic.Get: queue {} is not available", queue);
        return get_empty(channel_id);
    };
    let Some(shard_name) = topic.storage_name_list.get(&0).cloned() else {
        error!("AMQP Basic.Get: queue {} has no shard", queue);
        return get_empty(channel_id);
    };

    let claimed = claim_next_message(
        &ctx.storage_driver_manager,
        &ctx.storage_driver_manager
            .engine_storage_handler
            .client_pool,
        &tenant,
        queue,
        &shard_name,
        no_ack,
        connection_id,
        channel_id,
    )
    .await;

    let (record, msg_offset, index_offset) = match claimed {
        Ok(Some(claimed)) => claimed,
        Ok(None) => return get_empty(channel_id),
        Err(e) => {
            error!("AMQP Basic.Get failed for {}: {}", queue, e);
            return get_empty(channel_id);
        }
    };

    let delivery_tag = ctx
        .amqp_cache
        .get_channel(connection_id, channel_id)
        .map(|channel| channel.next_delivery_tag.fetch_add(1, Ordering::SeqCst))
        .unwrap_or(1);

    if !no_ack {
        if let Some(index_offset) = index_offset {
            ctx.amqp_cache.unacked().insert(
                (connection_id, channel_id, delivery_tag),
                UnackedEntry {
                    tenant: tenant.clone(),
                    queue: queue.to_string(),
                    offset: msg_offset,
                    index_offset,
                },
            );
        }
    }

    let redelivered = record
        .protocol_data
        .as_ref()
        .and_then(|pd| pd.amqp.as_ref())
        .map(|a| a.redelivered)
        .unwrap_or(false);
    let body = record.data.to_vec();
    let body_size = body.len() as u64;

    let get_ok_frame = AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::GetOk(GetOk {
            delivery_tag,
            redelivered,
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
            properties: properties_from_record(&record),
        }),
    );
    let body_frame = AMQPFrame::Body(channel_id, body);

    for frame in [get_ok_frame, header_frame, body_frame] {
        let wrapper = RobustMQPacketWrapper {
            protocol: RobustMQProtocol::AMQP,
            extend: RobustMQWrapperExtend::AMQP(AmqpWrapperExtend {}),
            packet: RobustMQPacket::AMQP(frame),
        };
        if let Err(e) = ctx
            .connection_manager
            .write_tcp_frame(connection_id, wrapper)
            .await
        {
            error!(connection_id, "AMQP Basic.Get write failed: {}", e);
            return None;
        }
    }

    None
}

#[allow(clippy::too_many_arguments)]
async fn claim_next_message(
    sdm: &Arc<StorageDriverManager>,
    client_pool: &Arc<grpc_clients::pool::ClientPool>,
    tenant: &str,
    queue: &str,
    shard_name: &str,
    no_ack: bool,
    connection_id: u64,
    channel_id: u16,
) -> Result<Option<(StorageRecord, u64, Option<u64>)>, CommonError> {
    let read_config = AdapterReadConfig::new();

    for _ in 0..MAX_CLAIM_ATTEMPTS {
        let current = offset::read_committed_offset(client_pool, tenant, queue, shard_name).await?;

        let mut offsets = HashMap::new();
        offsets.insert(shard_name.to_string(), current);
        let records = sdm
            .read_by_offset(tenant, queue, &offsets, &read_config)
            .await?;
        let Some(record) = records.into_iter().next() else {
            return Ok(None);
        };

        let msg_offset = record.metadata.offset;
        let new_offset = msg_offset + 1;

        let index_offset = if no_ack {
            None
        } else {
            Some(
                unacked_index::write_entry(
                    sdm,
                    tenant,
                    queue,
                    msg_offset,
                    connection_id,
                    channel_id,
                    broker_config().broker_id,
                )
                .await?,
            )
        };

        if offset::commit_offset_cas(client_pool, tenant, queue, shard_name, current, new_offset)
            .await?
        {
            return Ok(Some((record, msg_offset, index_offset)));
        }

        if let Some(index_offset) = index_offset {
            if let Err(e) = unacked_index::delete_entry(sdm, index_offset).await {
                warn!(
                    "AMQP Basic.Get: failed to clean up a stale index entry: {}",
                    e
                );
            }
        }
    }

    Err(CommonError::CommonError(format!(
        "AMQP Basic.Get: gave up claiming from {} after {} conflicting attempts",
        queue, MAX_CLAIM_ATTEMPTS
    )))
}

async fn process_settle(
    delivery_tag: Option<u64>,
    multiple: bool,
    requeue: bool,
    connection_id: u64,
    channel_id: u16,
    ctx: &BasicCtx,
) {
    let mut settled: Vec<(u64, UnackedEntry)> = Vec::new();
    for entry in ctx.amqp_cache.unacked().iter() {
        let &(conn, chan, tag) = entry.key();
        if conn != connection_id || chan != channel_id {
            continue;
        }
        let matches = match delivery_tag {
            Some(dt) => tag == dt || (multiple && tag <= dt),
            None => true,
        };
        if matches {
            settled.push((tag, entry.value().clone()));
        }
    }
    for (tag, _) in &settled {
        ctx.amqp_cache
            .unacked()
            .remove(&(connection_id, channel_id, *tag));
    }

    if requeue {
        for (_, entry) in &settled {
            if let Err(e) = requeue_message(
                &ctx.storage_driver_manager,
                &entry.tenant,
                &entry.queue,
                entry.offset,
                entry.index_offset,
            )
            .await
            {
                error!(
                    "AMQP: failed to requeue message from {}: {}",
                    entry.queue, e
                );
            }
        }
        return;
    }

    let mut by_queue: HashMap<(String, String), Vec<u64>> = HashMap::new();
    for (_, entry) in &settled {
        by_queue
            .entry((entry.tenant.clone(), entry.queue.clone()))
            .or_default()
            .push(entry.offset);
    }
    for ((tenant, queue), offsets) in by_queue {
        if let Err(e) = ctx
            .storage_driver_manager
            .delete_by_offsets(&tenant, &queue, &offsets)
            .await
        {
            error!(
                "AMQP: failed to delete settled messages from {}: {}",
                queue, e
            );
        }
    }
    for (_, entry) in &settled {
        if let Err(e) =
            unacked_index::delete_entry(&ctx.storage_driver_manager, entry.index_offset).await
        {
            warn!("AMQP: failed to delete index entry: {}", e);
        }
    }
}

pub(crate) async fn requeue_channel(connection_id: u64, channel_id: u16, ctx: &BasicCtx) {
    process_settle(None, false, true, connection_id, channel_id, ctx).await;
}

pub(crate) async fn requeue_connection(connection_id: u64, ctx: &BasicCtx) {
    let mut settled: Vec<((u64, u16, u64), UnackedEntry)> = Vec::new();
    for entry in ctx.amqp_cache.unacked().iter() {
        let key = *entry.key();
        if key.0 == connection_id {
            settled.push((key, entry.value().clone()));
        }
    }
    for (key, _) in &settled {
        ctx.amqp_cache.unacked().remove(key);
    }
    for (_, entry) in &settled {
        if let Err(e) = requeue_message(
            &ctx.storage_driver_manager,
            &entry.tenant,
            &entry.queue,
            entry.offset,
            entry.index_offset,
        )
        .await
        {
            error!(
                "AMQP: failed to requeue message from {} on connection close: {}",
                entry.queue, e
            );
        }
    }
}

pub fn process_basic(channel_id: u16, method: &AMQPMethod) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Qos(_) => process_qos(channel_id),
        AMQPMethod::Cancel(m) => process_cancel(channel_id, m.consumer_tag.as_str()),
        _ => None,
    }
}

pub fn process_confirm(channel_id: u16, method: &confirm::AMQPMethod) -> Option<AMQPFrame> {
    match method {
        confirm::AMQPMethod::Select(_) => process_confirm_select(channel_id),
        _ => None,
    }
}

fn process_qos(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::QosOk(QosOk {})),
    ))
}

fn process_cancel(channel_id: u16, consumer_tag: &str) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::CancelOk(CancelOk {
            consumer_tag: consumer_tag.into(),
        })),
    ))
}

fn process_confirm_select(channel_id: u16) -> Option<AMQPFrame> {
    use amq_protocol::protocol::confirm::{AMQPMethod as ConfirmMethod, SelectOk};
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Confirm(ConfirmMethod::SelectOk(SelectOk {})),
    ))
}
