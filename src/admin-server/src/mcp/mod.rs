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

pub mod error;
pub mod mq9;
pub mod protocol;

use crate::path::MCP_PATH;
use crate::state::HttpState;
use async_nats::Client;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use mq9::{
    agent::{DiscoverAgentsArgs, RegisterAgentArgs, UnregisterAgentArgs},
    mailbox::CreateMailboxArgs,
    message::{
        AckMessageArgs, DeleteMessageArgs, FetchMessagesArgs, QueryMailboxArgs, SendMessageArgs,
    },
};
use protocol::{McpRequest, McpResponse, ToolsListResult};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn mcp_route() -> Router<Arc<HttpState>> {
    Router::new()
        .route(MCP_PATH, post(handle_mcp))
        // MCP 2025-03-26: OAuth discovery endpoints — respond with no-auth required
        .route("/.well-known/oauth-protected-resource", get(oauth_protected_resource))
        .route("/.well-known/oauth-protected-resource/mcp", get(oauth_protected_resource))
        .route("/.well-known/oauth-authorization-server", get(|| async { StatusCode::NOT_FOUND }))
        .route("/.well-known/oauth-authorization-server/mcp", get(|| async { StatusCode::NOT_FOUND }))
        .route("/.well-known/openid-configuration", get(|| async { StatusCode::NOT_FOUND }))
        .route("/.well-known/openid-configuration/mcp", get(|| async { StatusCode::NOT_FOUND }))
        .route("/mcp/.well-known/openid-configuration", get(|| async { StatusCode::NOT_FOUND }))
        // OAuth dynamic client registration — return 400 to indicate registration is not supported
        .route("/register", post(|| async {
            (StatusCode::BAD_REQUEST, axum::Json(serde_json::json!({
                "error": "invalid_client_metadata",
                "error_description": "Dynamic client registration is not supported. No authentication required."
            })))
        }))
}

/// Declare that /mcp requires no OAuth authentication.
/// MCP clients check this endpoint to determine auth requirements.
async fn oauth_protected_resource(headers: HeaderMap) -> Json<Value> {
    let host = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("localhost:58080");
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http");

    Json(json!({
        "resource": format!("{proto}://{host}/mcp"),
        "bearer_methods_supported": [],
        "resource_documentation": "https://github.com/robustmq/robustmq"
    }))
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

        "initialize" => Ok(McpResponse::ok(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "robustmq-mq9-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )),

        "tools/list" => {
            let tools = mq9::mq9_tools();
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

            let nats_ctx = state
                .nats_context
                .as_ref()
                .ok_or("mq9 tools require nats-broker to be running")?;
            let nats_url = format!("nats://127.0.0.1:{}", nats_ctx.nats_tcp_port);
            let nats_client = async_nats::connect(&nats_url)
                .await
                .map_err(|e| format!("failed to connect to nats: {e}"))?;

            dispatch_tool_call(&nats_client, id, tool_name, args).await
        }

        _ => Ok(McpResponse::err(id, -32601, "Method not found")),
    }
}

async fn dispatch_tool_call(
    client: &Client,
    id: Option<Value>,
    tool_name: &str,
    args: Value,
) -> Result<McpResponse, Box<dyn std::error::Error + Send + Sync>> {
    let result = match tool_name {
        "mq9_create_mailbox" => {
            let a: CreateMailboxArgs = serde_json::from_value(args)?;
            mq9::mailbox::create_mailbox(client, a).await?
        }
        "mq9_send_message" => {
            let a: SendMessageArgs = serde_json::from_value(args)?;
            mq9::message::send_message(client, a).await?
        }
        "mq9_fetch_messages" => {
            let a: FetchMessagesArgs = serde_json::from_value(args)?;
            mq9::message::fetch_messages(client, a).await?
        }
        "mq9_ack_message" => {
            let a: AckMessageArgs = serde_json::from_value(args)?;
            mq9::message::ack_message(client, a).await?
        }
        "mq9_query_mailbox" => {
            let a: QueryMailboxArgs = serde_json::from_value(args)?;
            mq9::message::query_mailbox(client, a).await?
        }
        "mq9_register_agent" => {
            let a: RegisterAgentArgs = serde_json::from_value(args)?;
            mq9::agent::register_agent(client, a).await?
        }
        "mq9_discover_agents" => {
            let a: DiscoverAgentsArgs = serde_json::from_value(args)?;
            mq9::agent::discover_agents(client, a).await?
        }
        "mq9_delete_message" => {
            let a: DeleteMessageArgs = serde_json::from_value(args)?;
            mq9::message::delete_message(client, a).await?
        }
        "mq9_unregister_agent" => {
            let a: UnregisterAgentArgs = serde_json::from_value(args)?;
            mq9::agent::unregister_agent(client, a).await?
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
