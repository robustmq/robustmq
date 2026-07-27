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
    use crate::nats::common::{admin_client, nats_connect};
    use a2a_types::{AgentCapabilities, AgentCard, AgentInterface, AgentSkill};
    use admin_server::mq9::agent::AgentListReq;
    use async_nats::Client;
    use bytes::Bytes;
    use metadata_struct::mq9::agent::MQ9Agent;
    use mq9_core::command::Mq9Command;
    use mq9_core::protocol::{AgentRegisterReply, AgentRegisterReq, AgentUnregisterReply};
    use serde::Serialize;
    use std::time::Duration;
    use tokio::time::sleep;

    // Wait for raft write + broker cache notify to propagate
    const PROPAGATION_DELAY: u64 = 3;

    async fn register_agent(client: &Client, req: &AgentRegisterReq) -> AgentRegisterReply {
        let payload = Bytes::from(serde_json::to_string(req).unwrap());
        let subject = Mq9Command::AgentRegister.to_subject();
        let msg = client.request(subject, payload).await.unwrap();
        serde_json::from_slice::<AgentRegisterReply>(&msg.payload).unwrap()
    }

    async fn unregister_agent(client: &Client, name: &str) -> AgentUnregisterReply {
        #[derive(Serialize)]
        struct UnregisterReq<'a> {
            name: &'a str,
        }
        let payload = Bytes::from(serde_json::to_string(&UnregisterReq { name }).unwrap());
        let subject = Mq9Command::AgentUnregister.to_subject();
        let msg = client.request(subject, payload).await.unwrap();
        serde_json::from_slice::<AgentUnregisterReply>(&msg.payload).unwrap()
    }

    #[tokio::test]
    async fn mq9_agent_register_unregister_test() {
        let admin = admin_client();
        let nats = nats_connect().await;

        let agent_name = format!(
            "test-agent-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        );

        // ── build AgentCard ───────────────────────────────────────────────────
        let card = AgentCard {
            name: agent_name.clone(),
            description: "RobustMQ test agent".to_string(),
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
                id: "echo".to_string(),
                name: "Echo".to_string(),
                description: "Echoes the input back".to_string(),
                tags: vec!["echo".to_string()],
                examples: vec![],
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
        };

        // ── 1. register agent ─────────────────────────────────────────────────
        let req = AgentRegisterReq {
            name: agent_name.clone(),
            payload: serde_json::to_string(&card).unwrap(),
        };
        let reply = register_agent(&nats, &req).await;
        println!("register reply: {:?}", reply);
        assert!(reply.error.is_empty(), "register failed: {}", reply.error);

        // wait for raft write + cache notify to propagate
        sleep(Duration::from_secs(PROPAGATION_DELAY)).await;

        // ── 2. list via admin — agent must exist ──────────────────────────────
        let list_req = AgentListReq {
            name: Some(agent_name.clone()),
            ..Default::default()
        };
        let list = admin
            .get_agent_list::<_, Vec<MQ9Agent>>(&list_req)
            .await
            .unwrap();
        println!("agent list after register: {:#?}", list);
        assert_eq!(list.data.len(), 1, "expected 1 agent after register");
        assert_eq!(list.data[0].name, agent_name);

        // ── 3. unregister agent ───────────────────────────────────────────────
        let unregister_reply = unregister_agent(&nats, &agent_name).await;
        println!("unregister reply: {:?}", unregister_reply);
        assert!(
            unregister_reply.error.is_empty(),
            "unregister failed: {}",
            unregister_reply.error
        );

        // wait for raft delete + cache notify to propagate
        sleep(Duration::from_secs(PROPAGATION_DELAY)).await;

        // ── 4. list via admin — agent must be gone ────────────────────────────
        let list_after = admin
            .get_agent_list::<_, Vec<MQ9Agent>>(&list_req)
            .await
            .unwrap();
        println!("agent list after unregister: {:#?}", list_after);
        assert_eq!(
            list_after.data.len(),
            0,
            "agent should be removed after unregister"
        );
    }
}
