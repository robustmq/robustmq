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
use mq9_core::protocol::{
    AgentDiscoverReply, AgentRegisterReply, AgentReportReq, AgentUnregisterReply,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::mcp::error::McpToolError;

// ── register ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterAgentArgs {
    /// Agent name (unique identifier).
    pub name: String,
    /// Agent card payload (A2A AgentCard JSON serialized as a string, or any
    /// description of the agent's capabilities).
    pub payload: String,
}

pub async fn register_agent(
    client: &Client,
    args: RegisterAgentArgs,
) -> Result<Value, McpToolError> {
    let req = mq9_core::protocol::AgentRegisterReq {
        name: args.name.clone(),
        payload: args.payload,
    };
    let payload = Bytes::from(serde_json::to_string(&req)?);

    let msg = client
        .request(Mq9Command::AgentRegister.to_subject(), payload)
        .await
        .map_err(|e| McpToolError::BrokerError(e.to_string()))?;

    let reply: AgentRegisterReply = serde_json::from_slice(&msg.payload)?;
    if !reply.error.is_empty() {
        return Err(McpToolError::BrokerError(reply.error));
    }

    Ok(json!({ "name": args.name, "registered": true }))
}

// ── discover ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DiscoverAgentsArgs {
    /// Natural-language description or tag query (e.g. "tag:translation").
    /// The broker will match agents whose capabilities overlap with this query.
    pub query: Option<String>,
    /// Maximum number of agents to return (default 20).
    pub limit: Option<u32>,
}

pub async fn discover_agents(
    client: &Client,
    args: DiscoverAgentsArgs,
) -> Result<Value, McpToolError> {
    let body = json!({
        "query": args.query,
        "limit": args.limit.unwrap_or(20),
    });
    let payload = Bytes::from(body.to_string());

    let msg = client
        .request(Mq9Command::AgentDiscover.to_subject(), payload)
        .await
        .map_err(|e| McpToolError::BrokerError(e.to_string()))?;

    let reply: AgentDiscoverReply = serde_json::from_slice(&msg.payload)?;
    if !reply.error.is_empty() {
        return Err(McpToolError::BrokerError(reply.error));
    }

    Ok(json!({ "agents": reply.agents }))
}

// ── unregister ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UnregisterAgentArgs {
    pub name: String,
}

pub async fn unregister_agent(
    client: &Client,
    args: UnregisterAgentArgs,
) -> Result<Value, McpToolError> {
    let body = json!({ "name": args.name });
    let payload = Bytes::from(body.to_string());

    let msg = client
        .request(Mq9Command::AgentUnregister.to_subject(), payload)
        .await
        .map_err(|e| McpToolError::BrokerError(e.to_string()))?;

    let reply: AgentUnregisterReply = serde_json::from_slice(&msg.payload)?;
    if !reply.error.is_empty() {
        return Err(McpToolError::BrokerError(reply.error));
    }

    Ok(json!({ "name": args.name, "unregistered": true }))
}

// ── report (internal, not exposed to LLM) ────────────────────────────────────

pub async fn report_agent_status(
    client: &Client,
    name: String,
    report_info: Option<String>,
) -> Result<(), McpToolError> {
    let req = AgentReportReq { name, report_info };
    let payload = Bytes::from(serde_json::to_string(&req)?);

    client
        .publish(Mq9Command::AgentReport.to_subject(), payload)
        .await
        .map_err(|e| McpToolError::BrokerError(e.to_string()))?;

    Ok(())
}
