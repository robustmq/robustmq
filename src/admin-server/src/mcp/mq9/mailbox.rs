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

use async_nats::Client;
use bytes::Bytes;
use mq9_core::command::Mq9Command;
use mq9_core::protocol::{MailboxCreateReply, MailboxCreateReq};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::mcp::error::McpToolError;

#[derive(Debug, Deserialize)]
pub struct CreateMailboxArgs {
    /// Mailbox name. Must be lowercase. Auto-generated if omitted.
    pub name: Option<String>,
    /// TTL in seconds. Uses broker default when absent.
    pub ttl: Option<u64>,
    /// Optional human-readable description.
    pub desc: Option<String>,
}

pub async fn create_mailbox(
    client: &Client,
    args: CreateMailboxArgs,
) -> Result<Value, McpToolError> {
    let req = MailboxCreateReq {
        name: args.name,
        ttl: args.ttl,
        desc: args.desc,
    };
    let payload = Bytes::from(serde_json::to_string(&req)?);
    let subject = Mq9Command::MailboxCreate.to_subject();

    let msg = client
        .request(subject, payload)
        .await
        .map_err(|e| McpToolError::BrokerError(e.to_string()))?;

    let reply: MailboxCreateReply = serde_json::from_slice(&msg.payload)?;
    if !reply.error.is_empty() {
        return Err(McpToolError::BrokerError(reply.error));
    }

    Ok(json!({
        "mail_address": reply.mail_address,
        "created": true,
    }))
}
