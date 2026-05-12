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

use async_nats::{Client, HeaderMap, HeaderName, HeaderValue};
use bytes::Bytes;
use mq9_core::command::Mq9Command;
use mq9_core::protocol::{
    DeliverPolicy, MsgAckReply, MsgAckReq, MsgDeleteReply, MsgFetchConfig, MsgFetchReq,
    MsgQueryReply, MsgQueryReq, MsgSendReply,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::mcp::error::McpToolError;

// ── send ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SendMessageArgs {
    pub mail_address: String,
    /// Message body (UTF-8 string).
    pub payload: String,
    /// Priority: "normal" | "urgent" | "critical". Default "normal".
    pub priority: Option<String>,
}

pub async fn send_message(client: &Client, args: SendMessageArgs) -> Result<Value, McpToolError> {
    let subject = Mq9Command::MsgSend {
        mail_address: args.mail_address.clone(),
    }
    .to_subject();

    let mut headers = HeaderMap::new();
    if let Some(p) = &args.priority {
        headers.insert(
            HeaderName::from_static("mq9-priority"),
            HeaderValue::from(p.as_str()),
        );
    }

    let msg = client
        .request_with_headers(subject, headers, Bytes::from(args.payload))
        .await
        .map_err(|e| McpToolError::BrokerError(e.to_string()))?;

    let reply: MsgSendReply = serde_json::from_slice(&msg.payload)?;
    if !reply.error.is_empty() {
        return Err(McpToolError::BrokerError(reply.error));
    }

    Ok(json!({
        "msg_id": reply.msg_id,
        "mail_address": args.mail_address,
    }))
}

// ── fetch ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct FetchMessagesArgs {
    pub mail_address: String,
    /// Consumer group name. When provided, the broker tracks the read position per group
    /// so the next fetch resumes where the last ack left off (stateful consumption).
    /// When omitted, the broker uses a transient group — no position is saved and every
    /// call starts fresh from `reset_to` (stateless / one-shot consumption).
    pub group_name: Option<String>,
    /// Where to start reading. Only takes effect when there is no saved position,
    /// or when explicitly overriding with force reset.
    /// "earliest" | "latest" (default) | "time:<unix_ts>" | "id:<msg_id>"
    pub reset_to: Option<String>,
    /// Maximum number of messages to return (default 100).
    pub max_messages: Option<u32>,
}

pub async fn fetch_messages(
    client: &Client,
    args: FetchMessagesArgs,
) -> Result<Value, McpToolError> {
    let (deliver, from_time, from_id, force_deliver) = parse_reset_to(args.reset_to.as_deref());

    let req = MsgFetchReq {
        group_name: args.group_name,
        deliver,
        from_time,
        from_id,
        force_deliver,
        config: Some(MsgFetchConfig {
            num_msgs: args.max_messages,
            max_wait_ms: None,
        }),
    };

    let payload = Bytes::from(serde_json::to_string(&req)?);
    let subject = Mq9Command::MsgFetch {
        mail_address: args.mail_address,
    }
    .to_subject();

    let msg = client
        .request(subject, payload)
        .await
        .map_err(|e| McpToolError::BrokerError(e.to_string()))?;

    let reply: mq9_core::protocol::MsgFetchReply = serde_json::from_slice(&msg.payload)?;
    if !reply.error.is_empty() {
        return Err(McpToolError::BrokerError(reply.error));
    }

    let messages: Vec<Value> = reply
        .messages
        .into_iter()
        .map(|m| {
            json!({
                "msg_id":      m.msg_id,
                "payload":     m.payload,
                "priority":    m.priority,
                "create_time": m.create_time,
            })
        })
        .collect();

    Ok(json!({ "messages": messages }))
}

/// Parse the `reset_to` shorthand into low-level fetch parameters.
///
/// Supported values:
/// - omitted        → resume from saved position; if none, start from latest (no force reset)
/// - "earliest"     → force reset to the beginning of the mailbox
/// - "latest"       → force reset, only receive new messages from now
/// - "time:<ts>"    → force reset to the given Unix timestamp (seconds)
/// - "id:<msg_id>"  → force reset to the given msg_id
fn parse_reset_to(
    reset_to: Option<&str>,
) -> (DeliverPolicy, Option<u64>, Option<u64>, Option<bool>) {
    match reset_to {
        None => (DeliverPolicy::Latest, None, None, None),
        Some("earliest") => (DeliverPolicy::Earliest, None, None, Some(true)),
        Some("latest") => (DeliverPolicy::Latest, None, None, Some(true)),
        Some(s) => {
            if let Some(ts_str) = s.strip_prefix("time:") {
                if let Ok(ts) = ts_str.parse::<u64>() {
                    return (DeliverPolicy::FromTime, Some(ts), None, Some(true));
                }
            }
            if let Some(id_str) = s.strip_prefix("id:") {
                if let Ok(id) = id_str.parse::<u64>() {
                    return (DeliverPolicy::FromId, None, Some(id), Some(true));
                }
            }
            (DeliverPolicy::Latest, None, None, Some(true))
        }
    }
}

// ── ack ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AckMessageArgs {
    pub mail_address: String,
    pub group_name: String,
    /// The msg_id of the last successfully processed message.
    pub msg_id: u64,
}

pub async fn ack_message(client: &Client, args: AckMessageArgs) -> Result<Value, McpToolError> {
    let req = MsgAckReq {
        group_name: args.group_name,
        mail_address: args.mail_address.clone(),
        msg_id: args.msg_id,
    };
    let payload = Bytes::from(serde_json::to_string(&req)?);
    let subject = Mq9Command::MsgAck {
        mail_address: args.mail_address,
    }
    .to_subject();

    let msg = client
        .request(subject, payload)
        .await
        .map_err(|e| McpToolError::BrokerError(e.to_string()))?;

    let reply: MsgAckReply = serde_json::from_slice(&msg.payload)?;
    if !reply.error.is_empty() {
        return Err(McpToolError::BrokerError(reply.error));
    }

    Ok(json!({ "msg_id": args.msg_id, "acked": true }))
}

// ── query ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct QueryMailboxArgs {
    pub mail_address: String,
    /// Filter by message key (exact match).
    pub key: Option<String>,
    /// Filter by tags (messages must carry ALL specified tags).
    pub tags: Option<Vec<String>>,
    /// Only return messages created after this Unix timestamp (seconds).
    pub since: Option<u64>,
    /// Maximum number of messages to return (default 20).
    pub limit: Option<u64>,
}

pub async fn query_mailbox(client: &Client, args: QueryMailboxArgs) -> Result<Value, McpToolError> {
    let req = MsgQueryReq {
        key: args.key,
        tags: args.tags,
        since: args.since,
        limit: args.limit,
    };
    let payload = Bytes::from(serde_json::to_string(&req)?);
    let subject = Mq9Command::MsgQuery {
        mail_address: args.mail_address,
    }
    .to_subject();

    let msg = client
        .request(subject, payload)
        .await
        .map_err(|e| McpToolError::BrokerError(e.to_string()))?;

    let reply: MsgQueryReply = serde_json::from_slice(&msg.payload)?;
    if !reply.error.is_empty() {
        return Err(McpToolError::BrokerError(reply.error));
    }

    let messages: Vec<Value> = reply
        .messages
        .into_iter()
        .map(|m| {
            json!({
                "msg_id":      m.msg_id,
                "payload":     m.payload,
                "priority":    m.priority,
                "create_time": m.create_time,
            })
        })
        .collect();

    Ok(json!({ "messages": messages }))
}

// ── delete ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DeleteMessageArgs {
    pub mail_address: String,
    pub msg_id: u64,
}

pub async fn delete_message(
    client: &Client,
    args: DeleteMessageArgs,
) -> Result<Value, McpToolError> {
    let subject = Mq9Command::MsgDelete {
        mail_address: args.mail_address,
        msg_id: args.msg_id.to_string(),
    }
    .to_subject();

    let msg = client
        .request(subject, Bytes::new())
        .await
        .map_err(|e| McpToolError::BrokerError(e.to_string()))?;

    let reply: MsgDeleteReply = serde_json::from_slice(&msg.payload)?;
    if !reply.error.is_empty() {
        return Err(McpToolError::BrokerError(reply.error));
    }

    Ok(json!({ "msg_id": args.msg_id, "deleted": true }))
}
