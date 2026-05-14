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

#[cfg(test)]
mod tests {
    use crate::nats::common::nats_connect;
    use a2a_types::{AgentCapabilities, AgentCard, AgentInterface, AgentSkill};
    use async_nats::Client;
    use bytes::Bytes;
    use mq9_core::command::Mq9Command;
    use mq9_core::protocol::{
        AgentDiscoverReply, AgentDiscoverReq, AgentRegisterReply, AgentRegisterReq,
        AgentUnregisterReply,
    };
    use serde::Serialize;
    use std::time::Duration;
    use tokio::time::sleep;

    // Vector indexing + raft write + cache notify are async; wait long enough.
    const PROPAGATION_DELAY: u64 = 5;

    fn make_payment_agent(name: &str) -> AgentCard {
        AgentCard {
            name: name.to_string(),
            description:
                "Agent specialized in payment processing, invoices, and financial transactions"
                    .to_string(),
            version: "1.0.0".to_string(),
            supported_interfaces: vec![AgentInterface {
                url: "https://example.com/a2a/v1".to_string(),
                protocol_binding: "JSONRPC".to_string(),
                protocol_version: "0.3".to_string(),
                tenant: String::new(),
            }],
            capabilities: Some(AgentCapabilities {
                streaming: Some(false),
                push_notifications: Some(false),
                extended_agent_card: Some(false),
                extensions: vec![],
            }),
            default_input_modes: vec!["text/plain".to_string()],
            default_output_modes: vec!["text/plain".to_string()],
            skills: vec![AgentSkill {
                id: "payment".to_string(),
                name: "Payment".to_string(),
                description: "Process payments and generate invoices".to_string(),
                tags: vec![
                    "payment".to_string(),
                    "invoice".to_string(),
                    "billing".to_string(),
                ],
                examples: vec!["Pay invoice #1234".to_string()],
                input_modes: vec![],
                output_modes: vec![],
                security_requirements: vec![],
            }],
            provider: None,
            documentation_url: None,
            security_schemes: Default::default(),
            security_requirements: vec![],
            signatures: vec![],
            icon_url: None,
        }
    }

    async fn register_agent(client: &Client, name: &str, card: &AgentCard) -> AgentRegisterReply {
        let req = AgentRegisterReq {
            name: name.to_string(),
            payload: serde_json::to_string(card).unwrap(),
        };
        let payload = Bytes::from(serde_json::to_string(&req).unwrap());
        let msg = client
            .request(Mq9Command::AgentRegister.to_subject(), payload)
            .await
            .unwrap();
        serde_json::from_slice::<AgentRegisterReply>(&msg.payload).unwrap()
    }

    async fn unregister_agent(client: &Client, name: &str) -> AgentUnregisterReply {
        #[derive(Serialize)]
        struct UnregisterReq<'a> {
            name: &'a str,
        }
        let payload = Bytes::from(serde_json::to_string(&UnregisterReq { name }).unwrap());
        let msg = client
            .request(Mq9Command::AgentUnregister.to_subject(), payload)
            .await
            .unwrap();
        serde_json::from_slice::<AgentUnregisterReply>(&msg.payload).unwrap()
    }

    async fn discover(client: &Client, req: &AgentDiscoverReq) -> AgentDiscoverReply {
        let payload = Bytes::from(serde_json::to_string(req).unwrap());
        let msg = client
            .request(Mq9Command::AgentDiscover.to_subject(), payload)
            .await
            .unwrap();
        serde_json::from_slice::<AgentDiscoverReply>(&msg.payload).unwrap()
    }

    fn agent_name_in_reply(reply: &AgentDiscoverReply, name: &str) -> bool {
        reply.agents.iter().any(|a| {
            a.get("name")
                .and_then(|v| v.as_str())
                .map(|n| n == name)
                .unwrap_or(false)
        })
    }

    #[tokio::test]
    async fn mq9_agent_discover_text_test() {
        let nats = nats_connect().await;
        let agent_name = format!(
            "payment-agent-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        );
        let card = make_payment_agent(&agent_name);

        // ── 1. register ───────────────────────────────────────────────────────
        let reply = register_agent(&nats, &agent_name, &card).await;
        assert!(reply.error.is_empty(), "register failed: {}", reply.error);

        // wait for raft write + fts index to be ready
        sleep(Duration::from_secs(PROPAGATION_DELAY)).await;

        // ── 2. DISCOVER by text — must find the agent ─────────────────────────
        let found = discover(
            &nats,
            &AgentDiscoverReq {
                text: Some("payment invoice".to_string()),
                ..Default::default()
            },
        )
        .await;
        println!("discover (text) reply: {:?}", found);
        assert!(found.error.is_empty(), "discover error: {}", found.error);
        assert!(
            agent_name_in_reply(&found, &agent_name),
            "agent not found by text search"
        );

        // ── 3. unregister ─────────────────────────────────────────────────────
        let unregister_reply = unregister_agent(&nats, &agent_name).await;
        assert!(
            unregister_reply.error.is_empty(),
            "unregister failed: {}",
            unregister_reply.error
        );

        sleep(Duration::from_secs(PROPAGATION_DELAY)).await;

        // ── 4. DISCOVER by text — must not find the agent ─────────────────────
        let gone = discover(
            &nats,
            &AgentDiscoverReq {
                text: Some("payment invoice".to_string()),
                ..Default::default()
            },
        )
        .await;
        println!("discover (text) after unregister: {:?}", gone);
        assert!(gone.error.is_empty(), "discover error: {}", gone.error);
        assert!(
            !agent_name_in_reply(&gone, &agent_name),
            "agent should not be found after unregister"
        );
    }

    #[tokio::test]
    async fn mq9_agent_discover_semantic_test() {
        let nats = nats_connect().await;
        let agent_name = format!(
            "payment-agent-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        );
        let card = make_payment_agent(&agent_name);

        // ── 1. register ───────────────────────────────────────────────────────
        let reply = register_agent(&nats, &agent_name, &card).await;
        assert!(reply.error.is_empty(), "register failed: {}", reply.error);

        // wait for raft write + vector index to be ready
        sleep(Duration::from_secs(PROPAGATION_DELAY)).await;

        // ── 2. DISCOVER by semantic — must find the agent ─────────────────────
        let found = discover(
            &nats,
            &AgentDiscoverReq {
                semantic: Some("process a payment and generate invoice".to_string()),
                ..Default::default()
            },
        )
        .await;
        println!("discover (semantic) reply: {:?}", found);
        assert!(found.error.is_empty(), "discover error: {}", found.error);
        assert!(
            agent_name_in_reply(&found, &agent_name),
            "agent not found by semantic search"
        );

        // ── 3. unregister ─────────────────────────────────────────────────────
        let unregister_reply = unregister_agent(&nats, &agent_name).await;
        assert!(
            unregister_reply.error.is_empty(),
            "unregister failed: {}",
            unregister_reply.error
        );

        sleep(Duration::from_secs(PROPAGATION_DELAY)).await;

        // ── 4. DISCOVER by semantic — must not find the agent ─────────────────
        let gone = discover(
            &nats,
            &AgentDiscoverReq {
                semantic: Some("process a payment and generate invoice".to_string()),
                ..Default::default()
            },
        )
        .await;
        println!("discover (semantic) after unregister: {:?}", gone);
        assert!(gone.error.is_empty(), "discover error: {}", gone.error);
        assert!(
            !agent_name_in_reply(&gone, &agent_name),
            "agent should not be found after unregister"
        );
    }
}
