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

pub mod mailbox;
pub mod message;

use crate::mcp::protocol::Tool;
use serde_json::json;

/// Returns the mq9 MCP tools exposed to AI clients.
///
/// Only operations backed by the mq9 protocol layer are exposed:
/// - create_mailbox  (MailboxCreate)
/// - list_messages   (MailboxList)
/// - delete_message  (MailboxDelete)
/// - send_message    (MailboxMsg / publish)
pub fn mq9_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "mq9_create_mailbox".to_string(),
            description: "Create a new mq9 mailbox for a tenant.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tenant":  { "type": "string", "description": "Tenant name" },
                    "mail_address": { "type": "string", "description": "Mailbox identifier" },
                    "desc":    { "type": "string", "description": "Optional description" },
                    "public":  { "type": "boolean", "description": "Whether the mailbox is public (default false)" },
                    "ttl":     { "type": "integer", "description": "TTL in seconds (optional, uses broker default)" }
                },
                "required": ["tenant", "mail_address"]
            }),
        },
        Tool {
            name: "mq9_list_messages".to_string(),
            description: "List messages in a mailbox.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tenant":  { "type": "string",  "description": "Tenant name" },
                    "mail_address": { "type": "string",  "description": "Mailbox identifier" },
                    "offset":  { "type": "integer", "description": "Start offset for pagination (default 0)" },
                    "limit":   { "type": "integer", "description": "Maximum number of messages to return (default 20, max 500)" }
                },
                "required": ["tenant", "mail_address"]
            }),
        },
        Tool {
            name: "mq9_delete_message".to_string(),
            description: "Delete a specific message from a mailbox by its ID.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tenant":  { "type": "string",  "description": "Tenant name" },
                    "mail_address": { "type": "string",  "description": "Mailbox identifier" },
                    "msg_id":  { "type": "integer", "description": "Message offset / ID to delete" }
                },
                "required": ["tenant", "mail_address", "msg_id"]
            }),
        },
        Tool {
            name: "mq9_send_message".to_string(),
            description: "Publish a message to a mailbox.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tenant":   { "type": "string", "description": "Tenant name" },
                    "mail_address":  { "type": "string", "description": "Mailbox identifier" },
                    "priority": { "type": "string", "description": "Message priority (e.g. \"normal\", \"high\", default \"normal\")" },
                    "payload":  { "type": "string", "description": "Message body (UTF-8 string)" }
                },
                "required": ["tenant", "mail_address", "payload"]
            }),
        },
    ]
}
