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

//! MCP handlers for mq9 message operations.
//!
//! | Tool               | Backed by                                    |
//! |--------------------|----------------------------------------------|
//! | mq9_list_messages  | StorageDriverManager::read_by_tag            |
//! | mq9_delete_message | StorageDriverManager::delete_by_offset       |
//! | mq9_send_message   | StorageDriverManager::write (via MessageStorage) |

use crate::state::NatsContext;
use bytes::Bytes;
use common_base::error::common::CommonError;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::record::{StorageRecordProtocolData, StorageRecordProtocolDataMq9};
use nats_broker::nats::subscribe::subject_message_tag;
use nats_broker::storage::message::MessageStorage;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use storage_adapter::driver::StorageDriverManager;

const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 500;

#[derive(Debug, Deserialize)]
pub struct ListMessagesArgs {
    pub tenant: String,
    pub mail_address: String,
    /// Start read offset (inclusive). Default 0.
    pub offset: Option<u64>,
    /// Maximum number of messages to return. Default 20, max 500.
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMessageArgs {
    pub tenant: String,
    pub mail_address: String,
    /// Message offset / ID returned by list or send.
    pub msg_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageArgs {
    pub tenant: String,
    pub mail_address: String,
    /// Message priority string (e.g. "normal", "high"). Default "normal".
    pub priority: Option<String>,
    /// Message body as a UTF-8 string.
    pub payload: String,
}

/// List messages in a mailbox using read_by_tag for tag-scoped pagination.
pub async fn list_messages(
    _ctx: &NatsContext,
    driver: &StorageDriverManager,
    args: ListMessagesArgs,
) -> Result<Value, CommonError> {
    let limit = args.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as u64;
    let start_offset = args.offset.unwrap_or(0);
    let tag = subject_message_tag(&args.tenant, &args.mail_address);

    let read_config = AdapterReadConfig {
        max_record_num: limit,
        max_size: 1024 * 1024 * 30,
    };

    // Seed per-shard offsets so read starts from the requested position.
    // We don't know shard names up front, so we use an empty map and let
    // the driver start from 0, then skip records below start_offset.
    let offsets: HashMap<String, u64> = HashMap::new();
    let _ = start_offset; // driver reads from shard start; caller can paginate via returned msg_ids

    let records = driver
        .read_by_tag(
            &args.tenant,
            &args.mail_address,
            &tag,
            &offsets,
            &read_config,
        )
        .await?;

    let messages: Vec<Value> = records
        .into_iter()
        .filter(|r| r.metadata.offset >= start_offset)
        .take(limit as usize)
        .map(|r| {
            let priority = r
                .protocol_data
                .as_ref()
                .and_then(|pd| pd.mq9.as_ref())
                .map(|m| m.priority.clone())
                .unwrap_or_default();
            json!({
                "msg_id":      r.metadata.offset,
                "payload":     String::from_utf8_lossy(&r.data),
                "priority":    priority,
                "create_time": r.metadata.create_t,
            })
        })
        .collect();

    Ok(json!({
        "mail_address":  args.mail_address,
        "messages": messages,
    }))
}

/// Delete a specific message by its offset (msg_id).
pub async fn delete_message(
    _ctx: &NatsContext,
    driver: &StorageDriverManager,
    args: DeleteMessageArgs,
) -> Result<Value, CommonError> {
    driver
        .delete_by_offset(&args.tenant, &args.mail_address, args.msg_id)
        .await?;

    Ok(json!({ "msg_id": args.msg_id, "deleted": true }))
}

/// Publish a message to a mailbox via MessageStorage.
pub async fn send_message(
    ctx: &NatsContext,
    driver: &StorageDriverManager,
    args: SendMessageArgs,
) -> Result<Value, CommonError> {
    // Verify the mailbox exists in cache.
    if ctx
        .cache_manager
        .get_mail(&args.tenant, &args.mail_address)
        .is_none()
    {
        return Err(CommonError::CommonError(format!(
            "mailbox {} does not exist",
            args.mail_address
        )));
    }

    let priority_str = args.priority.unwrap_or_else(|| "normal".to_string());
    let tag = subject_message_tag(&args.tenant, &args.mail_address);

    let record = AdapterWriteRecord::new(args.mail_address.clone(), Bytes::from(args.payload))
        .with_tags(vec![tag])
        .with_protocol_data(Some(StorageRecordProtocolData {
            mq9: Some(StorageRecordProtocolDataMq9 {
                priority: priority_str,
                header: None,
            }),
            nats: None,
            mqtt: None,
        }));

    let storage = MessageStorage::new(driver.clone().into());
    let offsets = storage
        .write(&args.tenant, &args.mail_address, vec![record])
        .await?;

    let msg_id = offsets.into_iter().next().unwrap_or(0);
    Ok(json!({ "msg_id": msg_id, "mail_address": args.mail_address }))
}
