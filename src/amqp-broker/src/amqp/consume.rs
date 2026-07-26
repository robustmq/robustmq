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

use std::sync::atomic::Ordering;

use amq_protocol::frame::AMQPFrame;
use amq_protocol::protocol::basic::{AMQPMethod, CancelOk, ConsumeOk, GetEmpty, GetOk};
use amq_protocol::protocol::AMQPClass;
use common_base::error::common::CommonError;
use common_base::uuid::unique_id;
use common_config::broker::broker_config;
use grpc_clients::broker::common::call::broker_fetch_amqp_queue_message;
use metadata_struct::storage::record::StorageRecord;
use protocol::broker::broker::FetchAmqpQueueMessageRequest;
use tracing::error;

use crate::amqp::basic::{properties_from_record, BasicCtx};
use crate::amqp::channel::channel_error_close;
use crate::amqp::queue;
use crate::core::cache::UnackedEntry;
use crate::core::consume_group::{add_consume_member, remove_consume_member};
use crate::core::frame::build_basic_content_frames;
use crate::push;
use crate::storage::offset::OffsetStorage;

pub(crate) async fn process_consume(
    channel_id: u16,
    queue: &str,
    consumer_tag: &str,
    no_ack: bool,
    exclusive: bool,
    connection_id: u64,
    ctx: &BasicCtx,
) -> Option<Vec<AMQPFrame>> {
    let tenant = ctx.amqp_cache.tenant_for(connection_id);
    let broker_id = broker_config().broker_id;

    // Empty consumer-tag means the broker assigns one; without this, two
    // consumers that both leave it blank would collide on the same member key.
    let consumer_tag = if consumer_tag.is_empty() {
        unique_id()
    } else {
        consumer_tag.to_string()
    };

    if queue::declare_amqp_queue(&ctx.storage_driver_manager, &tenant, queue)
        .await
        .is_none()
    {
        error!("AMQP Basic.Consume: queue {} is not available", queue);
        return Some(vec![channel_error_close(
            channel_id,
            541,
            "INTERNAL_ERROR",
            60,
            20,
        )]);
    }

    // Exclusive: this consumer wants sole ownership of the queue, and no
    // other consumer may already be attached (nor may an existing exclusive
    // consumer be joined by anyone else). Read from the locally replicated
    // share-group member cache, same source used to compute Queue.Declare's
    // consumer_count.
    let existing = ctx
        .storage_driver_manager
        .broker_cache
        .get_share_group_members(&tenant, queue);
    let existing_exclusive = existing.iter().any(|m| {
        matches!(&m.params, metadata_struct::mqtt::share_group::ShareGroupParams::AMQP(d) if d.exclusive)
    });
    if existing_exclusive || (exclusive && !existing.is_empty()) {
        return Some(vec![channel_error_close(
            channel_id,
            403,
            "ACCESS_REFUSED",
            60,
            20,
        )]);
    }

    if let Err(e) = add_consume_member(
        &ctx.client_pool,
        &tenant,
        queue,
        connection_id,
        broker_id,
        channel_id,
        &consumer_tag,
        no_ack,
        exclusive,
    )
    .await
    {
        error!(
            "AMQP Basic.Consume: failed to register consumer for {}: {}",
            queue, e
        );
        return Some(vec![channel_error_close(
            channel_id,
            541,
            "INTERNAL_ERROR",
            60,
            20,
        )]);
    }
    ctx.amqp_cache
        .register_consumer(connection_id, channel_id, &consumer_tag, queue);

    Some(vec![AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::ConsumeOk(ConsumeOk {
            consumer_tag: consumer_tag.into(),
        })),
    )])
}

pub(crate) async fn process_cancel(
    channel_id: u16,
    consumer_tag: &str,
    connection_id: u64,
    ctx: &BasicCtx,
) -> Option<Vec<AMQPFrame>> {
    if let Some(reg) = ctx
        .amqp_cache
        .remove_consumer(connection_id, channel_id, consumer_tag)
    {
        if let Err(e) = remove_consume_member(
            &ctx.client_pool,
            broker_config().broker_id,
            connection_id,
            channel_id,
            consumer_tag,
        )
        .await
        {
            error!(
                "AMQP Basic.Cancel: failed to deregister consumer for {}: {}",
                reg.queue, e
            );
        }
    }

    Some(vec![AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::CancelOk(CancelOk {
            consumer_tag: consumer_tag.into(),
        })),
    )])
}

/// Deregisters every consumer registered on `channel_id`, e.g. on
/// `Channel.Close`.
pub(crate) async fn cancel_channel_consumers(connection_id: u64, channel_id: u16, ctx: &BasicCtx) {
    let broker_id = broker_config().broker_id;
    for (consumer_tag, _reg) in ctx
        .amqp_cache
        .remove_consumers_by_channel(connection_id, channel_id)
    {
        if let Err(e) = remove_consume_member(
            &ctx.client_pool,
            broker_id,
            connection_id,
            channel_id,
            &consumer_tag,
        )
        .await
        {
            error!(
                "AMQP: failed to deregister consumer on channel close: {}",
                e
            );
        }
    }
}

/// Deregisters every consumer registered on `connection_id`, e.g. on
/// `Connection.Close`.
pub(crate) async fn cancel_connection_consumers(connection_id: u64, ctx: &BasicCtx) {
    let broker_id = broker_config().broker_id;
    for (channel_id, consumer_tag, _reg) in
        ctx.amqp_cache.remove_consumers_by_connection(connection_id)
    {
        if let Err(e) = remove_consume_member(
            &ctx.client_pool,
            broker_id,
            connection_id,
            channel_id,
            &consumer_tag,
        )
        .await
        {
            error!(
                "AMQP: failed to deregister consumer on connection close: {}",
                e
            );
        }
    }
}

fn get_empty(channel_id: u16) -> Option<Vec<AMQPFrame>> {
    Some(vec![AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::GetEmpty(GetEmpty {})),
    )])
}

fn get_internal_error(channel_id: u16) -> Option<Vec<AMQPFrame>> {
    Some(vec![channel_error_close(
        channel_id,
        541,
        "INTERNAL_ERROR",
        60,
        70,
    )])
}

pub(crate) async fn process_get(
    channel_id: u16,
    queue: &str,
    no_ack: bool,
    connection_id: u64,
    ctx: &BasicCtx,
) -> Option<Vec<AMQPFrame>> {
    let tenant = ctx.amqp_cache.tenant_for(connection_id);

    let Some(topic) = queue::declare_amqp_queue(&ctx.storage_driver_manager, &tenant, queue).await
    else {
        error!("AMQP Basic.Get: queue {} is not available", queue);
        return get_internal_error(channel_id);
    };
    let Some(shard_name) = topic.storage_name_list.get(&0).cloned() else {
        error!("AMQP Basic.Get: queue {} has no shard", queue);
        return get_internal_error(channel_id);
    };

    let leader_broker_id = match push::resolve_queue_leader(
        &ctx.client_pool,
        &ctx.storage_driver_manager.broker_cache,
        &tenant,
        queue,
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            error!(
                "AMQP Basic.Get: failed to resolve leader for {}: {}",
                queue, e
            );
            return get_internal_error(channel_id);
        }
    };

    let claimed = if push::is_self(leader_broker_id) {
        claim_locally(
            ctx,
            &tenant,
            queue,
            &shard_name,
            no_ack,
            connection_id,
            channel_id,
        )
        .await
    } else {
        claim_via_leader(
            ctx,
            leader_broker_id,
            &tenant,
            queue,
            &shard_name,
            no_ack,
            connection_id,
            channel_id,
        )
        .await
    };

    let (record, msg_offset, index_offset) = match claimed {
        Ok(Some(v)) => v,
        Ok(None) => return get_empty(channel_id),
        Err(e) => {
            error!("AMQP Basic.Get failed for {}: {}", queue, e);
            return get_internal_error(channel_id);
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

    Some(build_basic_content_frames(
        channel_id,
        get_ok_frame,
        body,
        properties_from_record(&record),
    ))
}

#[allow(clippy::too_many_arguments)]
async fn claim_locally(
    ctx: &BasicCtx,
    tenant: &str,
    queue: &str,
    shard_name: &str,
    no_ack: bool,
    connection_id: u64,
    channel_id: u16,
) -> Result<Option<(StorageRecord, u64, Option<u64>)>, CommonError> {
    let offset_storage = OffsetStorage::new(
        ctx.storage_driver_manager
            .engine_storage_handler
            .client_pool
            .clone(),
    );
    offset_storage
        .claim_and_track(
            &ctx.push_manager,
            &ctx.storage_driver_manager,
            tenant,
            queue,
            shard_name,
            no_ack,
            connection_id,
            channel_id,
            broker_config().broker_id,
        )
        .await
}

#[allow(clippy::too_many_arguments)]
async fn claim_via_leader(
    ctx: &BasicCtx,
    leader_broker_id: u64,
    tenant: &str,
    queue: &str,
    shard_name: &str,
    no_ack: bool,
    connection_id: u64,
    channel_id: u16,
) -> Result<Option<(StorageRecord, u64, Option<u64>)>, CommonError> {
    let Some(node) = ctx
        .storage_driver_manager
        .broker_cache
        .node_lists
        .get(&leader_broker_id)
    else {
        return Err(CommonError::CommonError(format!(
            "AMQP Basic.Get: leader broker {} for queue {} is not known to this node",
            leader_broker_id, queue
        )));
    };
    let addr = node.grpc_addr.clone();
    drop(node);

    let request = FetchAmqpQueueMessageRequest {
        tenant: tenant.to_string(),
        queue: queue.to_string(),
        shard_name: shard_name.to_string(),
        connect_id: connection_id,
        channel_id: channel_id as u32,
        no_ack,
        requester_broker_id: broker_config().broker_id,
    };
    let reply = broker_fetch_amqp_queue_message(&ctx.client_pool, &[addr], request).await?;
    if !reply.has_message {
        return Ok(None);
    }
    let record = StorageRecord::decode(&reply.record)?;
    Ok(Some((record, reply.offset, reply.index_offset)))
}
