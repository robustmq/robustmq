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

use amq_protocol::frame::{AMQPContentHeader, AMQPFrame};
use amq_protocol::protocol::basic::{AMQPMethod, Ack, Nack, Return};
use amq_protocol::protocol::AMQPClass;
use common_base::error::common::CommonError;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::record::{StorageRecordProtocolData, StorageRecordProtocolDataAmqp};
use storage_adapter::driver::StorageDriverManager;
use tracing::{debug, error};

use crate::amqp::basic::{properties_from_protocol_data, properties_to_protocol_data, BasicCtx};
use crate::amqp::{queue, route};
use crate::core::cache::PendingPublish;
use crate::core::frame::build_basic_content_frames;

/// Content Header frame: carries body_size for the Basic.Publish that preceded
/// it. A zero-length body means the message is already complete.
pub(crate) async fn process_content_header_full(
    connection_id: u64,
    channel_id: u16,
    class_id: u16,
    header: &AMQPContentHeader,
    ctx: &BasicCtx,
) -> Option<Vec<AMQPFrame>> {
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
            return finalize_publish(channel_id, pending, ctx).await;
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
) -> Option<Vec<AMQPFrame>> {
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
            return finalize_publish(channel_id, pending, ctx).await;
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
pub(crate) async fn finalize_publish(
    channel_id: u16,
    pending: PendingPublish,
    ctx: &BasicCtx,
) -> Option<Vec<AMQPFrame>> {
    if pending.exchange.is_empty() && pending.routing_key.is_empty() {
        tracing::warn!("AMQP Basic.Publish with empty routing key ignored on the default exchange");
        return None;
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
        // Unroutable: still "handled" from the publisher's point of view, so
        // a Confirm-mode publisher gets acked even though there's no queue.
        let mut frames = confirm_frames(channel_id, pending.confirm_seqno, true);
        if pending.mandatory {
            frames.extend(build_basic_return_frames(channel_id, &pending));
        } else {
            debug!(
                "AMQP Basic.Publish unroutable (exchange={}, routing_key={}), dropped",
                pending.exchange, pending.routing_key
            );
        }
        return (!frames.is_empty()).then_some(frames);
    }

    let mut all_ok = true;
    for queue_name in &queues {
        let ok = write_to_queue(
            &ctx.storage_driver_manager,
            &pending.tenant,
            queue_name,
            pending.body.clone(),
            &pending.properties,
        )
        .await;
        all_ok &= ok;
    }
    let frames = confirm_frames(channel_id, pending.confirm_seqno, all_ok);
    (!frames.is_empty()).then_some(frames)
}

/// Builds the Confirm-mode Basic.Ack/Basic.Nack for one publish, or nothing
/// if the channel isn't in Confirm.Select mode.
fn confirm_frames(channel_id: u16, confirm_seqno: Option<u64>, ok: bool) -> Vec<AMQPFrame> {
    let Some(delivery_tag) = confirm_seqno else {
        return Vec::new();
    };
    let frame = if ok {
        AMQPFrame::Method(
            channel_id,
            AMQPClass::Basic(AMQPMethod::Ack(Ack {
                delivery_tag,
                multiple: false,
            })),
        )
    } else {
        AMQPFrame::Method(
            channel_id,
            AMQPClass::Basic(AMQPMethod::Nack(Nack {
                delivery_tag,
                multiple: false,
                requeue: false,
            })),
        )
    };
    vec![frame]
}

async fn write_to_queue(
    sdm: &Arc<StorageDriverManager>,
    tenant: &str,
    queue_name: &str,
    body: Vec<u8>,
    properties: &StorageRecordProtocolDataAmqp,
) -> bool {
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
        Ok(_) => true,
        Err(CommonError::TopicNotFoundInBrokerCache(_, _)) => {
            // Published to a queue that was never explicitly declared (common
            // with the default exchange): declare it on the fly, then retry.
            if queue::declare_amqp_queue(sdm, tenant, queue_name)
                .await
                .is_some()
            {
                match sdm.write(tenant, queue_name, &[record], 1).await {
                    Ok(_) => true,
                    Err(e) => {
                        error!(
                            "AMQP Basic.Publish retry write failed for {}: {}",
                            queue_name, e
                        );
                        false
                    }
                }
            } else {
                error!(
                    "AMQP Basic.Publish dropped: queue {} does not exist and could not be created",
                    queue_name
                );
                false
            }
        }
        Err(e) => {
            error!("AMQP Basic.Publish write failed for {}: {}", queue_name, e);
            false
        }
    }
}

/// Builds an unroutable `mandatory` publish's reply to its publisher, per
/// spec: Basic.Return followed by the message's own content header and body.
fn build_basic_return_frames(channel_id: u16, pending: &PendingPublish) -> Vec<AMQPFrame> {
    let return_frame = AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::Return(Return {
            reply_code: 312,
            reply_text: "NO_ROUTE".into(),
            exchange: pending.exchange.clone().into(),
            routing_key: pending.routing_key.clone().into(),
        })),
    );
    build_basic_content_frames(
        channel_id,
        return_frame,
        pending.body.clone(),
        properties_from_protocol_data(&pending.properties),
    )
}
