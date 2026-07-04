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

use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;
use crate::{core::error::NatsBrokerError, nats::subscribe::subject_message_tag};
use bytes::Bytes;
use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use mq9_core::protocol::{MsgItem, MsgQueryReply, MsgQueryReq};
use std::collections::HashMap;

const MAX_QUERY_MSGS: usize = 100;

pub async fn process_query(
    ctx: &NatsProcessContext,
    mail_address: &str,
    payload: &Bytes,
) -> Result<MsgQueryReply, NatsBrokerError> {
    let req: MsgQueryReq = if payload.is_empty() {
        MsgQueryReq::default()
    } else {
        serde_json::from_slice(payload)
            .map_err(|e| NatsBrokerError::CommonError(format!("invalid query request: {}", e)))?
    };

    let limit = req.limit.unwrap_or(MAX_QUERY_MSGS as u64) as usize;
    let limit = limit.min(MAX_QUERY_MSGS);

    let messages = if let Some(key) = &req.key {
        query_by_key(ctx, mail_address, key, limit).await?
    } else if let Some(tags) = &req.tags {
        query_by_tags(ctx, mail_address, tags, limit).await?
    } else if let Some(since) = &req.since {
        query_by_since(ctx, mail_address, *since, limit).await?
    } else {
        query_all(ctx, mail_address, limit).await?
    };

    Ok(MsgQueryReply {
        error: String::new(),
        messages,
    })
}

async fn query_by_key(
    ctx: &NatsProcessContext,
    mail_address: &str,
    key: &str,
    limit: usize,
) -> Result<Vec<MsgItem>, NatsBrokerError> {
    let tenant = get_tenant();
    let sk = super::scoped_key(&tenant, mail_address, key);
    let key_refs: Vec<&[u8]> = vec![sk.as_bytes()];
    let result = ctx
        .storage_driver_manager
        .read_by_keys(&tenant, mail_address, &key_refs)
        .await
        .map_err(NatsBrokerError::from)?;

    let records = result.into_values().flatten();
    let messages = records.take(limit).map(to_msg_item).collect();

    Ok(messages)
}

async fn query_by_tags(
    ctx: &NatsProcessContext,
    mail_address: &str,
    tags: &[String],
    limit: usize,
) -> Result<Vec<MsgItem>, NatsBrokerError> {
    if tags.is_empty() {
        return query_by_since(ctx, mail_address, 0, limit).await;
    }

    let tenant = get_tenant();
    let read_config = AdapterReadConfig {
        max_record_num: limit as u64,
        max_size: 1024 * 1024 * 30,
    };

    let mut seen_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let mut messages = Vec::new();

    for tag in tags {
        let st = super::scoped_tag(&tenant, mail_address, tag);
        let records = ctx
            .storage_driver_manager
            .read_by_tag(&tenant, mail_address, &st, &HashMap::new(), &read_config)
            .await
            .map_err(NatsBrokerError::from)?;

        for record in records {
            if seen_ids.insert(record.metadata.offset) {
                messages.push(to_msg_item(record));
            }
        }
    }

    Ok(messages)
}

async fn query_all(
    ctx: &NatsProcessContext,
    mail_address: &str,
    limit: usize,
) -> Result<Vec<MsgItem>, NatsBrokerError> {
    let tenant = get_tenant();
    let system_tag = subject_message_tag(&tenant, mail_address);
    let read_config = AdapterReadConfig {
        max_record_num: limit as u64,
        max_size: 1024 * 1024 * 30,
    };
    let records = ctx
        .storage_driver_manager
        .read_by_tag(
            &tenant,
            mail_address,
            &system_tag,
            &HashMap::new(),
            &read_config,
        )
        .await
        .map_err(NatsBrokerError::from)?;
    Ok(records.into_iter().map(to_msg_item).collect())
}

async fn query_by_since(
    ctx: &NatsProcessContext,
    mail_address: &str,
    since: u64,
    limit: usize,
) -> Result<Vec<MsgItem>, NatsBrokerError> {
    let tenant = get_tenant();
    let read_config = AdapterReadConfig {
        max_record_num: limit as u64,
        max_size: 1024 * 1024 * 30,
    };

    // mq9 mail topics are always single-partition, so partition 0 is the only entry.
    let start_offset = ctx
        .storage_driver_manager
        .get_offset_by_timestamp(
            &tenant,
            mail_address,
            since,
            AdapterOffsetStrategy::Earliest,
        )
        .await
        .map_err(NatsBrokerError::from)?
        .get(&0)
        .copied()
        .unwrap_or(0);

    let mut offsets: HashMap<String, u64> = HashMap::new();
    offsets.insert(mail_address.to_string(), start_offset);

    let records = ctx
        .storage_driver_manager
        .read_by_offset(&tenant, mail_address, &offsets, &read_config)
        .await
        .map_err(NatsBrokerError::from)?;

    Ok(records.into_iter().map(to_msg_item).collect())
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
