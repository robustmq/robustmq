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

use amq_protocol::frame::AMQPFrame;
use amq_protocol::protocol::basic::{AMQPMethod, AMQPProperties, QosOk, RecoverOk};
use amq_protocol::protocol::confirm;
use amq_protocol::protocol::confirm::SelectOk as ConfirmSelectOk;
use amq_protocol::protocol::AMQPClass;
use amq_protocol::types::{AMQPValue, FieldTable};
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::record::{StorageRecord, StorageRecordProtocolDataAmqp};
use storage_adapter::driver::StorageDriverManager;
use tracing::{error, warn};

use crate::amqp::consume::{
    cancel_channel_consumers, cancel_connection_consumers, process_cancel, process_consume,
    process_get,
};
use crate::amqp::route;
use crate::core::cache::{AmqpCacheManager, PendingPublish, UnackedEntry};
use crate::core::recovery::requeue_message;
use crate::core::unacked_index;
use crate::push::AmqpPushManager;

/// Maps the wire-level AMQPProperties from a Content Header frame onto the
/// shape stored alongside the message, so redelivery can reconstruct them
/// instead of always sending an empty property set.
pub(crate) fn properties_to_protocol_data(
    properties: &AMQPProperties,
) -> StorageRecordProtocolDataAmqp {
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
pub(crate) fn properties_from_record(record: &StorageRecord) -> AMQPProperties {
    match record
        .protocol_data
        .as_ref()
        .and_then(|pd| pd.amqp.as_ref())
    {
        Some(amqp) => properties_from_protocol_data(amqp),
        None => AMQPProperties::default(),
    }
}

pub(crate) fn properties_from_protocol_data(
    amqp: &StorageRecordProtocolDataAmqp,
) -> AMQPProperties {
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
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub amqp_cache: Arc<AmqpCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub push_manager: Arc<AmqpPushManager>,
}

pub(crate) async fn process_basic_full(
    channel_id: u16,
    method: &AMQPMethod,
    connection_id: u64,
    ctx: &BasicCtx,
) -> Option<Vec<AMQPFrame>> {
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
            Some(vec![AMQPFrame::Method(
                channel_id,
                AMQPClass::Basic(AMQPMethod::RecoverOk(RecoverOk {})),
            )])
        }
        AMQPMethod::Consume(consume) => {
            process_consume(
                channel_id,
                consume.queue.as_str(),
                consume.consumer_tag.as_str(),
                consume.no_ack,
                connection_id,
                ctx,
            )
            .await
        }
        AMQPMethod::Cancel(cancel) => {
            process_cancel(channel_id, cancel.consumer_tag.as_str(), connection_id, ctx).await
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
        other => process_basic(channel_id, other).map(|f| vec![f]),
    }
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
    cancel_channel_consumers(connection_id, channel_id, ctx).await;
}

pub(crate) async fn requeue_connection(connection_id: u64, ctx: &BasicCtx) {
    cancel_connection_consumers(connection_id, ctx).await;
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

fn process_confirm_select(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Confirm(confirm::AMQPMethod::SelectOk(ConfirmSelectOk {})),
    ))
}
