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
use std::time::Duration;

use amq_protocol::frame::AMQPFrame;
use amq_protocol::protocol::basic::{AMQPMethod, ConsumeOk, Deliver, GetEmpty, GetOk};
use amq_protocol::protocol::AMQPClass;
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use protocol::robust::{
    AmqpWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use tokio::time::sleep;
use tracing::error;

use crate::amqp::basic::{properties_from_record, BasicCtx};
use crate::amqp::channel::channel_error_close;
use crate::amqp::queue;
use crate::core::cache::UnackedEntry;
use crate::core::frame::build_basic_content_frames;
use crate::storage::offset::OffsetStorage;

pub(crate) async fn process_consume(
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
                        let frames = build_basic_content_frames(
                            channel_id,
                            deliver_frame,
                            body,
                            properties_from_record(record),
                        );

                        let wrapper = RobustMQPacketWrapper {
                            protocol: RobustMQProtocol::AMQP,
                            extend: RobustMQWrapperExtend::AMQP(AmqpWrapperExtend {}),
                            packet: RobustMQPacket::AMQP(frames),
                        };
                        if let Err(e) = cm.write_tcp_frame(connection_id, wrapper).await {
                            error!(connection_id, "AMQP Deliver write failed: {}", e);
                            return;
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

    let offset_storage = OffsetStorage::new(
        ctx.storage_driver_manager
            .engine_storage_handler
            .client_pool
            .clone(),
    );
    let claimed = offset_storage
        .read_next_message(
            &ctx.storage_driver_manager,
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
