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

use crate::core::error::NatsBrokerError;
use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;
use crate::nats::subscribe::subject_message_tag;
use bytes::Bytes;
use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use mq9_core::protocol::{
    DeliverPolicy, MsgAckReply, MsgAckReq, MsgFetchReply, MsgFetchReq, MsgItem,
};
use std::collections::HashMap;
use storage_adapter::consumer::StartOffsetStrategy;
use storage_adapter::consumer_priority::PriorityGroupConsumer;
use uuid::Uuid;

const DEFAULT_NUM_MSGS: u32 = 100;
const DEFAULT_MAX_WAIT_MS: u64 = 500;

pub async fn process_fetch(
    ctx: &NatsProcessContext,
    mail_address: &str,
    payload: &Bytes,
) -> Result<MsgFetchReply, NatsBrokerError> {
    let req: MsgFetchReq =
        serde_json::from_slice(payload).map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;

    let tenant = get_tenant();
    let num_msgs = req
        .config
        .as_ref()
        .and_then(|c| c.num_msgs)
        .unwrap_or(DEFAULT_NUM_MSGS);
    let max_wait_ms = req
        .config
        .as_ref()
        .and_then(|c| c.max_wait_ms)
        .unwrap_or(DEFAULT_MAX_WAIT_MS);

    let (consumer, stateful) = match &req.group_name {
        Some(group_name) => {
            let c = PriorityGroupConsumer::new_manual(
                ctx.storage_driver_manager.clone(),
                group_name.clone(),
            );
            let group_exists = !ctx
                .storage_driver_manager
                .get_offset_by_group(&tenant, &format!("{}-normal", group_name))
                .await
                .map_err(NatsBrokerError::from)?
                .is_empty();
            if req.force_deliver.unwrap_or(false) && group_exists {
                let shard_offsets = force_reset_offset(ctx, &tenant, mail_address, &req).await?;
                c.set_current_offsets(&tenant, mail_address, &shard_offsets);
            } else {
                let strategy = deliver_to_strategy(&req);
                c.set_start_offset_strategy(strategy).await;
            }
            (c, true)
        }
        None => {
            let tmp_group = Uuid::new_v4().to_string();
            let c =
                PriorityGroupConsumer::new_manual(ctx.storage_driver_manager.clone(), tmp_group);
            let strategy = deliver_to_strategy(&req);
            c.set_start_offset_strategy(strategy).await;
            (c, false)
        }
    };
    let _ = stateful; // offset commit is controlled by process_ack, not here

    let base_tag = subject_message_tag(&tenant, mail_address);
    let read_config = AdapterReadConfig {
        max_record_num: num_msgs as u64,
        max_size: 1024 * 1024 * 30,
    };

    let records = consumer
        .next_messages_by_tags(&tenant, mail_address, &base_tag, &read_config)
        .await
        .map_err(NatsBrokerError::from)?;

    if records.is_empty() && max_wait_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(max_wait_ms)).await;
    }

    let messages = records.into_iter().map(to_msg_item).collect();
    Ok(MsgFetchReply {
        error: String::new(),
        messages,
    })
}

/// Compute the target offset from the deliver strategy and commit it to the group,
/// so that the consumer resumes from that position regardless of any prior offset.
async fn force_reset_offset(
    ctx: &NatsProcessContext,
    tenant: &str,
    mail_address: &str,
    req: &MsgFetchReq,
) -> Result<HashMap<String, u64>, NatsBrokerError> {
    let storage = &ctx.storage_driver_manager;

    let shard_offsets: HashMap<String, u64> = match &req.deliver {
        DeliverPolicy::Earliest => storage
            .list_storage_resource(tenant, mail_address)
            .await
            .map_err(NatsBrokerError::from)?
            .into_values()
            .map(|d| (d.shard_name, d.offset.start_offset))
            .collect(),
        DeliverPolicy::Latest => storage
            .list_storage_resource(tenant, mail_address)
            .await
            .map_err(NatsBrokerError::from)?
            .into_values()
            .map(|d| (d.shard_name, d.offset.end_offset))
            .collect(),
        DeliverPolicy::FromTime => {
            let ts = req.from_time.unwrap_or(0);
            let offsets = storage
                .get_offset_by_timestamp(tenant, mail_address, ts, AdapterOffsetStrategy::Earliest)
                .await
                .map_err(NatsBrokerError::from)?;
            storage
                .list_storage_resource(tenant, mail_address)
                .await
                .map_err(NatsBrokerError::from)?
                .into_iter()
                .map(|(partition, d)| {
                    let offset = offsets.get(&partition).copied().unwrap_or(0);
                    let o = offset.max(d.offset.start_offset).min(d.offset.end_offset);
                    (d.shard_name, o)
                })
                .collect()
        }
        DeliverPolicy::FromId => {
            let from_id = req.from_id.unwrap_or(0);
            storage
                .list_storage_resource(tenant, mail_address)
                .await
                .map_err(NatsBrokerError::from)?
                .into_values()
                .map(|d| {
                    let o = from_id.max(d.offset.start_offset).min(d.offset.end_offset);
                    (d.shard_name, o)
                })
                .collect()
        }
    };

    let reset_consumer = PriorityGroupConsumer::new_manual(
        ctx.storage_driver_manager.clone(),
        req.group_name.clone().unwrap_or_default(),
    );
    reset_consumer.stage_offsets(tenant, mail_address, &shard_offsets);
    reset_consumer
        .commit()
        .await
        .map_err(NatsBrokerError::from)?;

    Ok(shard_offsets)
}

fn deliver_to_strategy(req: &MsgFetchReq) -> StartOffsetStrategy {
    match &req.deliver {
        DeliverPolicy::Earliest => StartOffsetStrategy::Earliest,
        DeliverPolicy::Latest => StartOffsetStrategy::Latest,
        DeliverPolicy::FromTime => StartOffsetStrategy::ByStartTime(req.from_time.unwrap_or(0)),
        DeliverPolicy::FromId => StartOffsetStrategy::ByStartOffset(req.from_id.unwrap_or(0)),
    }
}

fn to_msg_item(record: metadata_struct::storage::record::StorageRecord) -> MsgItem {
    let (priority, header) = record
        .protocol_data
        .as_ref()
        .and_then(|pd| pd.mq9.as_ref())
        .map(|mq9| {
            (
                mq9.priority.clone(),
                mq9.header.as_ref().map(|h| h.to_vec()),
            )
        })
        .unwrap_or_default();

    MsgItem {
        msg_id: record.metadata.offset,
        payload: String::from_utf8_lossy(&record.data).into_owned(),
        priority,
        header,
        create_time: record.metadata.create_t,
    }
}

pub async fn process_ack(
    ctx: &NatsProcessContext,
    _mail_address: &str,
    payload: &Bytes,
) -> Result<MsgAckReply, NatsBrokerError> {
    let req: MsgAckReq =
        serde_json::from_slice(payload).map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;

    let tenant = get_tenant();

    let topic = ctx
        .storage_driver_manager
        .broker_cache
        .get_topic_by_name(&tenant, &req.mail_address)
        .ok_or_else(|| {
            NatsBrokerError::CommonError(format!("mailbox not found: {}", req.mail_address))
        })?;

    // Stage msg_id (the offset of the consumed message) for all priority groups.
    // commit() will persist msg_id+1 so the next FETCH starts after this message.
    let shard_offsets: HashMap<String, u64> = topic
        .storage_name_list
        .into_values()
        .map(|shard_name| (shard_name, req.msg_id))
        .collect();

    let consumer =
        PriorityGroupConsumer::new_manual(ctx.storage_driver_manager.clone(), req.group_name);
    consumer.stage_offsets(&tenant, &req.mail_address, &shard_offsets);
    consumer.commit().await.map_err(NatsBrokerError::from)?;

    Ok(MsgAckReply {
        error: String::new(),
    })
}
