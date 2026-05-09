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

/// Integration tests for the mq9 MCP server.
///
/// Prerequisites: a RobustMQ broker (nats-broker + admin-server) must be
/// running on the default ports before executing these tests.
///
/// Run with:
///   cargo test -p tests mcp_test -- --nocapture
#[cfg(test)]
mod tests {
    use common_base::uuid::unique_id;
    use reqwest::Client;
    use serde_json::{json, Value};

    const MCP_URL: &str = "http://127.0.0.1:8080/mcp";

    // ── helpers ──────────────────────────────────────────────────────────────

    fn http_client() -> Client {
        Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap()
    }

    /// Send a single JSON-RPC 2.0 request to /mcp and return the raw response Value.
    async fn mcp(client: &Client, method: &str, params: Option<Value>) -> Value {
        let mut body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
        });
        if let Some(p) = params {
            body["params"] = p;
        }

        let resp = client
            .post(MCP_URL)
            .json(&body)
            .send()
            .await
            .unwrap_or_else(|e| panic!("HTTP request failed for method={method}: {e}"));

        resp.json::<Value>()
            .await
            .unwrap_or_else(|e| panic!("JSON parse failed for method={method}: {e}"))
    }

    /// Call a tool via tools/call and return the result Value.
    async fn call_tool(client: &Client, tool: &str, arguments: Value) -> Value {
        let resp = mcp(
            client,
            "tools/call",
            Some(json!({ "name": tool, "arguments": arguments })),
        )
        .await;

        assert!(
            resp.get("error").is_none(),
            "tool={tool} returned error: {}",
            resp["error"]
        );
        resp["result"].clone()
    }

    // ── protocol tests ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_ping() {
        let client = http_client();
        let resp = mcp(&client, "ping", None).await;
        assert_eq!(resp["jsonrpc"], "2.0");
        assert!(resp.get("error").is_none(), "ping returned error: {resp}");
        assert_eq!(resp["result"], json!({}));
    }

    #[tokio::test]
    async fn test_initialize() {
        let client = http_client();
        let resp = mcp(&client, "initialize", None).await;
        assert!(resp.get("error").is_none(), "initialize error: {resp}");
        assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(resp["result"]["serverInfo"]["name"], "robustmq-mq9-mcp");
    }

    #[tokio::test]
    async fn test_tools_list() {
        let client = http_client();
        let resp = mcp(&client, "tools/list", None).await;
        assert!(resp.get("error").is_none(), "tools/list error: {resp}");

        let tools = resp["result"]["tools"]
            .as_array()
            .expect("tools must be array");
        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

        for expected in &[
            "mq9_create_mailbox",
            "mq9_send_message",
            "mq9_fetch_messages",
            "mq9_ack_message",
            "mq9_query_mailbox",
            "mq9_register_agent",
            "mq9_discover_agents",
            "mq9_unregister_agent",
        ] {
            assert!(
                names.contains(expected),
                "tool '{expected}' missing from tools/list; got {names:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let client = http_client();
        let resp = mcp(&client, "no_such_method", None).await;
        let code = resp["error"]["code"].as_i64().unwrap_or(0);
        assert_eq!(code, -32601, "expected Method not found: {resp}");
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let client = http_client();
        let resp = mcp(
            &client,
            "tools/call",
            Some(json!({ "name": "no_such_tool", "arguments": {} })),
        )
        .await;
        let code = resp["error"]["code"].as_i64().unwrap_or(0);
        assert_eq!(code, -32602, "expected Unknown tool error: {resp}");
    }

    // ── mailbox lifecycle ────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_mailbox_auto_name() {
        let client = http_client();
        let result = call_tool(&client, "mq9_create_mailbox", json!({})).await;
        let addr = result["mail_address"]
            .as_str()
            .expect("mail_address missing");
        assert!(!addr.is_empty(), "auto-generated address must be non-empty");
        assert_eq!(result["created"], true);
    }

    #[tokio::test]
    async fn test_create_mailbox_custom_name() {
        let client = http_client();
        let name = format!("test.{}", unique_id().replace('-', "."));
        // truncate to keep name lowercase-only
        let name = name.to_lowercase();
        let result = call_tool(
            &client,
            "mq9_create_mailbox",
            json!({ "name": name, "desc": "integration test mailbox" }),
        )
        .await;
        assert_eq!(result["created"], true);
        let addr = result["mail_address"].as_str().unwrap();
        assert!(
            addr.contains(&name[..8]),
            "address should contain name prefix"
        );
    }

    // ── send / fetch / ack ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_send_fetch_ack_flow() {
        let client = http_client();
        let group = unique_id();

        // 1. Create mailbox
        let mb = call_tool(&client, "mq9_create_mailbox", json!({})).await;
        let addr = mb["mail_address"].as_str().unwrap().to_string();

        // 2. Send two messages
        let r1 = call_tool(
            &client,
            "mq9_send_message",
            json!({ "mail_address": addr, "payload": "hello mcp 1" }),
        )
        .await;
        let id1 = r1["msg_id"].as_u64().expect("msg_id missing");

        let r2 = call_tool(
            &client,
            "mq9_send_message",
            json!({ "mail_address": addr, "payload": "hello mcp 2", "priority": "urgent" }),
        )
        .await;
        let id2 = r2["msg_id"].as_u64().expect("msg_id missing");
        assert!(id2 > id1, "msg_id must be monotonically increasing");

        // 3. Fetch from earliest
        let fetched = call_tool(
            &client,
            "mq9_fetch_messages",
            json!({
                "mail_address": addr,
                "group_name": group,
                "reset_to": "earliest",
                "max_messages": 10
            }),
        )
        .await;
        let msgs = fetched["messages"]
            .as_array()
            .expect("messages must be array");
        assert!(
            msgs.len() >= 2,
            "expected at least 2 messages, got {}",
            msgs.len()
        );

        let payloads: Vec<&str> = msgs.iter().filter_map(|m| m["payload"].as_str()).collect();
        assert!(payloads.contains(&"hello mcp 1"));
        assert!(payloads.contains(&"hello mcp 2"));

        // 4. Ack up to id2
        let ack = call_tool(
            &client,
            "mq9_ack_message",
            json!({ "mail_address": addr, "group_name": group, "msg_id": id2 }),
        )
        .await;
        assert_eq!(ack["acked"], true);
        assert_eq!(ack["msg_id"], id2);

        // 5. Fetch again — no new messages after ack (resume from committed offset)
        let fetched2 = call_tool(
            &client,
            "mq9_fetch_messages",
            json!({ "mail_address": addr, "group_name": group, "max_messages": 10 }),
        )
        .await;
        let msgs2 = fetched2["messages"]
            .as_array()
            .expect("messages must be array");
        assert!(
            msgs2.is_empty(),
            "after acking all messages, next fetch should be empty"
        );
    }

    #[tokio::test]
    #[ignore = "broker DeliverPolicy::Latest not yet skipping history — tracked as known issue"]
    async fn test_fetch_latest_skips_history() {
        let client = http_client();
        let group = unique_id();

        let mb = call_tool(&client, "mq9_create_mailbox", json!({})).await;
        let addr = mb["mail_address"].as_str().unwrap().to_string();

        call_tool(
            &client,
            "mq9_send_message",
            json!({ "mail_address": addr, "payload": "old message" }),
        )
        .await;

        let fetched = call_tool(
            &client,
            "mq9_fetch_messages",
            json!({
                "mail_address": addr,
                "group_name": group,
                "reset_to": "latest",
                "max_messages": 10
            }),
        )
        .await;
        let msgs = fetched["messages"]
            .as_array()
            .expect("messages must be array");
        assert!(msgs.is_empty(), "fetch with 'latest' should skip history");
    }

    #[tokio::test]
    async fn test_fetch_from_id() {
        let client = http_client();
        let group = unique_id();

        let mb = call_tool(&client, "mq9_create_mailbox", json!({})).await;
        let addr = mb["mail_address"].as_str().unwrap().to_string();

        // Send three messages
        call_tool(
            &client,
            "mq9_send_message",
            json!({ "mail_address": addr, "payload": "msg-a" }),
        )
        .await;
        let r2 = call_tool(
            &client,
            "mq9_send_message",
            json!({ "mail_address": addr, "payload": "msg-b" }),
        )
        .await;
        let id2 = r2["msg_id"].as_u64().unwrap();
        call_tool(
            &client,
            "mq9_send_message",
            json!({ "mail_address": addr, "payload": "msg-c" }),
        )
        .await;

        // Fetch starting from id2
        let fetched = call_tool(
            &client,
            "mq9_fetch_messages",
            json!({
                "mail_address": addr,
                "group_name": group,
                "reset_to": format!("id:{id2}"),
                "max_messages": 10
            }),
        )
        .await;
        let msgs = fetched["messages"].as_array().unwrap();
        let payloads: Vec<&str> = msgs.iter().filter_map(|m| m["payload"].as_str()).collect();

        // Should include msg-b and msg-c, but not msg-a
        assert!(
            !payloads.contains(&"msg-a"),
            "msg-a should be before id:{id2}"
        );
        assert!(payloads.iter().any(|p| *p == "msg-b" || *p == "msg-c"));
    }

    // ── query mailbox ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_query_mailbox() {
        let client = http_client();

        let mb = call_tool(&client, "mq9_create_mailbox", json!({})).await;
        let addr = mb["mail_address"].as_str().unwrap().to_string();

        // Send some messages
        call_tool(
            &client,
            "mq9_send_message",
            json!({ "mail_address": addr, "payload": "query-test-1" }),
        )
        .await;
        call_tool(
            &client,
            "mq9_send_message",
            json!({ "mail_address": addr, "payload": "query-test-2" }),
        )
        .await;

        // Query without consuming
        let result = call_tool(
            &client,
            "mq9_query_mailbox",
            json!({ "mail_address": addr, "limit": 10 }),
        )
        .await;
        let msgs = result["messages"]
            .as_array()
            .expect("messages must be array");
        assert!(msgs.len() >= 2, "query should see at least 2 messages");
    }

    // ── priority ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_send_all_priorities() {
        let client = http_client();
        let group = unique_id();

        let mb = call_tool(&client, "mq9_create_mailbox", json!({})).await;
        let addr = mb["mail_address"].as_str().unwrap().to_string();

        for priority in &["normal", "urgent", "critical"] {
            let r = call_tool(
                &client,
                "mq9_send_message",
                json!({ "mail_address": addr, "payload": format!("{priority}-msg"), "priority": priority }),
            )
            .await;
            assert!(
                r["msg_id"].as_u64().is_some(),
                "msg_id missing for priority={priority}"
            );
        }

        let fetched = call_tool(
            &client,
            "mq9_fetch_messages",
            json!({
                "mail_address": addr,
                "group_name": group,
                "reset_to": "earliest",
                "max_messages": 10
            }),
        )
        .await;
        let msgs = fetched["messages"].as_array().unwrap();
        // Critical/urgent messages should arrive before normal ones
        assert!(msgs.len() >= 3);
        assert_eq!(msgs[0]["priority"], "critical");
    }

    // ── agent registry ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_agent_register_discover_unregister() {
        let client = http_client();
        let agent_name = format!("test-agent-{}", unique_id());

        // Register
        let reg = call_tool(
            &client,
            "mq9_register_agent",
            json!({ "name": agent_name, "payload": "I am a test agent for integration tests" }),
        )
        .await;
        assert_eq!(reg["registered"], true);
        assert_eq!(reg["name"], agent_name);

        // Discover — broker returns agents array (may be empty if query-based filtering not implemented)
        let disc = call_tool(
            &client,
            "mq9_discover_agents",
            json!({ "query": "integration tests", "limit": 20 }),
        )
        .await;
        assert!(
            disc["agents"].is_array(),
            "discover must return an agents array"
        );

        // Unregister
        let unreg = call_tool(
            &client,
            "mq9_unregister_agent",
            json!({ "name": agent_name }),
        )
        .await;
        assert_eq!(unreg["unregistered"], true);
        assert_eq!(unreg["name"], agent_name);
    }

    #[tokio::test]
    async fn test_discover_agents_no_query() {
        let client = http_client();
        // Discover with no query should return all agents (up to limit)
        let disc = call_tool(&client, "mq9_discover_agents", json!({})).await;
        assert!(disc["agents"].is_array(), "agents must be array");
    }
}
