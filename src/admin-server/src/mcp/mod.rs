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

//! MCP (Model Context Protocol) server for RobustMQ Admin.
//!
//! Exposes a single `POST /mcp` endpoint that speaks JSON-RPC 2.0.
//! AI agents (Dify, Claude, etc.) discover available tools via `tools/list`
//! and invoke them via `tools/call`.
//!
//! Protocol subdirectories:
//! - `mq9/`   — mq9 mailbox operations
//! - `mqtt/`  — (future) MQTT operations
//! - `kafka/` — (future) Kafka-compatible operations

pub mod mq9;
pub mod protocol;

use crate::state::HttpState;
use axum::{extract::State, routing::post, Json, Router};
use mq9::{
    mailbox::CreateMailboxArgs,
    message::{DeleteMessageArgs, ListMessagesArgs, SendMessageArgs},
};
use protocol::{McpRequest, McpResponse, ToolsListResult};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn mcp_route() -> Router<Arc<HttpState>> {
    Router::new().route("/mcp", post(handle_mcp))
}

async fn handle_mcp(
    State(state): State<Arc<HttpState>>,
    Json(req): Json<McpRequest>,
) -> Json<McpResponse> {
    let id = req.id.clone();
    let resp = dispatch(state, req).await;
    Json(resp.unwrap_or_else(|e| McpResponse::err(id, -32603, e.to_string())))
}

async fn dispatch(
    state: Arc<HttpState>,
    req: McpRequest,
) -> Result<McpResponse, Box<dyn std::error::Error + Send + Sync>> {
    let id = req.id.clone();

    match req.method.as_str() {
        "ping" => Ok(McpResponse::ok(id, json!({}))),

        "initialize" => {
            let result = json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "robustmq-admin-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            });
            Ok(McpResponse::ok(id, result))
        }

        "tools/list" => {
            let tools = mq9::mq9_tools();
            // Future: append mqtt_tools(), kafka_tools(), …
            let result = serde_json::to_value(ToolsListResult { tools })?;
            Ok(McpResponse::ok(id, result))
        }

        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let tool_name = params
                .get("name")
                .and_then(Value::as_str)
                .ok_or("missing 'name' in tools/call params")?;
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or(Value::Object(Default::default()));

            dispatch_tool_call(&state, id, tool_name, args).await
        }

        _ => Ok(McpResponse::err(id, -32601, "Method not found")),
    }
}

async fn dispatch_tool_call(
    state: &Arc<HttpState>,
    id: Option<Value>,
    tool_name: &str,
    args: Value,
) -> Result<McpResponse, Box<dyn std::error::Error + Send + Sync>> {
    // All mq9 tools require the nats context.
    let nats_ctx = state
        .nats_context
        .as_ref()
        .ok_or("mq9 operations require nats-broker to be running")?;
    let driver = state.storage_driver_manager.as_ref();

    let result = match tool_name {
        "mq9_create_mailbox" => {
            let a: CreateMailboxArgs = serde_json::from_value(args)?;
            mq9::mailbox::create_mailbox(nats_ctx, a).await?
        }
        "mq9_list_messages" => {
            let a: ListMessagesArgs = serde_json::from_value(args)?;
            mq9::message::list_messages(nats_ctx, driver, a).await?
        }
        "mq9_delete_message" => {
            let a: DeleteMessageArgs = serde_json::from_value(args)?;
            mq9::message::delete_message(nats_ctx, driver, a).await?
        }
        "mq9_send_message" => {
            let a: SendMessageArgs = serde_json::from_value(args)?;
            mq9::message::send_message(nats_ctx, driver, a).await?
        }
        _ => {
            return Ok(McpResponse::err(
                id,
                -32602,
                format!("Unknown tool: {tool_name}"),
            ))
        }
    };

    Ok(McpResponse::ok(id, result))
}
