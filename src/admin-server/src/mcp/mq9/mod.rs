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

pub mod agent;
pub mod mailbox;
pub mod message;

use crate::mcp::protocol::Tool;
use serde_json::json;

pub fn mq9_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "mq9_create_mailbox".to_string(),
            description: "Create a new mq9 mailbox (inbox). Use this before sending or receiving messages. The mailbox name must be lowercase letters, digits, and dots. If name is omitted the broker generates one.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Mailbox name (lowercase, dots allowed). Auto-generated if omitted."
                    },
                    "ttl": {
                        "type": "integer",
                        "description": "Time-to-live in seconds. Uses broker default when absent."
                    },
                    "desc": {
                        "type": "string",
                        "description": "Optional human-readable description."
                    }
                },
                "required": []
            }),
        },
        Tool {
            name: "mq9_send_message".to_string(),
            description: "Send a message to a mailbox. Use this to deliver a message to another agent or to yourself.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mail_address": {
                        "type": "string",
                        "description": "Destination mailbox address."
                    },
                    "payload": {
                        "type": "string",
                        "description": "Message body (UTF-8 string). Can be plain text or JSON."
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["normal", "urgent", "critical"],
                        "description": "Message priority. Default is 'normal'. Use 'urgent' or 'critical' for time-sensitive messages."
                    }
                },
                "required": ["mail_address", "payload"]
            }),
        },
        Tool {
            name: "mq9_fetch_messages".to_string(),
            description: "Fetch messages from a mailbox. This is the primary way to consume messages. After processing, call mq9_ack_message to confirm delivery. The consumer position is tracked per group_name, so different groups consume independently.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mail_address": {
                        "type": "string",
                        "description": "Mailbox address to fetch from."
                    },
                    "group_name": {
                        "type": "string",
                        "description": "Consumer group identifier. Use a stable name (e.g. your agent ID) to track your read position across calls."
                    },
                    "reset_to": {
                        "type": "string",
                        "description": "Where to start reading. Omit to resume from the last acked position. Supported values: 'earliest' (re-read from the beginning), 'latest' (skip history, only new messages), 'time:<unix_seconds>' (start from a specific timestamp, e.g. 'time:1746000000'), 'id:<msg_id>' (start from a specific message, e.g. 'id:42')."
                    },
                    "max_messages": {
                        "type": "integer",
                        "description": "Maximum number of messages to return. Default 100."
                    }
                },
                "required": ["mail_address", "group_name"]
            }),
        },
        Tool {
            name: "mq9_ack_message".to_string(),
            description: "Acknowledge that messages up to msg_id have been processed. The next fetch call will resume after this message.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mail_address": {
                        "type": "string",
                        "description": "Mailbox address."
                    },
                    "group_name": {
                        "type": "string",
                        "description": "Consumer group name (must match the group used in fetch)."
                    },
                    "msg_id": {
                        "type": "integer",
                        "description": "ID of the last successfully processed message."
                    }
                },
                "required": ["mail_address", "group_name", "msg_id"]
            }),
        },
        Tool {
            name: "mq9_query_mailbox".to_string(),
            description: "Inspect messages in a mailbox without advancing the consumer position. Use this to peek at messages without consuming them, or to search by tag or time range.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mail_address": {
                        "type": "string",
                        "description": "Mailbox address."
                    },
                    "key": {
                        "type": "string",
                        "description": "Filter by message key (exact match)."
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Filter by tags. Only messages carrying all specified tags are returned."
                    },
                    "since": {
                        "type": "integer",
                        "description": "Only return messages created after this Unix timestamp (seconds)."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of messages to return. Default 20."
                    }
                },
                "required": ["mail_address"]
            }),
        },
        Tool {
            name: "mq9_register_agent".to_string(),
            description: "Register this agent in the mq9 agent registry so other agents can discover it. Call this once at startup with a description of the agent's capabilities.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Unique agent name."
                    },
                    "payload": {
                        "type": "string",
                        "description": "Agent capability description (plain text or A2A AgentCard JSON as a string)."
                    }
                },
                "required": ["name", "payload"]
            }),
        },
        Tool {
            name: "mq9_discover_agents".to_string(),
            description: "Find agents registered in the mq9 registry. Use this to locate agents with specific capabilities before sending them a message.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural-language capability query (e.g. 'translation agent') or tag query (e.g. 'tag:translation')."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of agents to return. Default 20."
                    }
                },
                "required": []
            }),
        },
        Tool {
            name: "mq9_unregister_agent".to_string(),
            description: "Remove this agent from the registry. Call this when the agent is shutting down.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Agent name to unregister."
                    }
                },
                "required": ["name"]
            }),
        },
    ]
}
