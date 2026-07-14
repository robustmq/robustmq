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
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    use a2a_types::{AgentCapabilities, AgentCard, AgentInterface, AgentSkill};
    use common_base::tools::now_second;
    use common_base::uuid::unique_id;
    use grpc_clients::meta::mq9::call::{
        placement_create_mq9_agent, placement_delete_mq9_agent, placement_list_mq9_agent,
        placement_search_mq9_agent,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mq9::agent::MQ9Agent;
    use protocol::meta::meta_service_mq9::{
        CreateAgentRequest, DeleteAgentRequest, ListAgentRequest, SearchAgentRequest,
    };

    use crate::common::get_placement_addr;

    fn make_agent_card(name: &str, description: &str) -> AgentCard {
        AgentCard {
            name: name.to_string(),
            description: description.to_string(),
            version: "1.0.0".to_string(),
            supported_interfaces: vec![AgentInterface {
                url: "https://example.com/a2a".to_string(),
                protocol_binding: "JSONRPC".to_string(),
                protocol_version: "1.0".to_string(),
                tenant: String::new(),
            }],
            capabilities: Some(AgentCapabilities {
                streaming: Some(false),
                push_notifications: Some(false),
                extensions: vec![],
                extended_agent_card: Some(false),
            }),
            skills: vec![AgentSkill {
                id: "search".to_string(),
                name: "Search".to_string(),
                description: description.to_string(),
                tags: vec!["search".to_string(), "query".to_string()],
                examples: vec!["Find information about X".to_string()],
                input_modes: vec![],
                output_modes: vec![],
                security_requirements: vec![],
            }],
            default_input_modes: vec!["text/plain".to_string()],
            default_output_modes: vec!["text/plain".to_string()],
            provider: None,
            documentation_url: None,
            security_schemes: Default::default(),
            security_requirements: vec![],
            signatures: vec![],
            icon_url: None,
        }
    }

    fn make_mq9_agent(tenant: &str, card: &AgentCard) -> MQ9Agent {
        MQ9Agent {
            tenant: tenant.to_string(),
            name: card.name.clone(),
            agent_info: serde_json::to_string(card).unwrap(),
            create_time: now_second(),
        }
    }

    async fn register_agent(
        client_pool: &Arc<ClientPool>,
        addrs: &[String],
        tenant: &str,
        card: &AgentCard,
    ) {
        let agent = make_mq9_agent(tenant, card);
        placement_create_mq9_agent(
            client_pool,
            addrs,
            CreateAgentRequest {
                tenant: tenant.to_string(),
                content: agent.encode().unwrap(),
            },
        )
        .await
        .unwrap();
    }

    async fn delete_agent(
        client_pool: &Arc<ClientPool>,
        addrs: &[String],
        tenant: &str,
        name: &str,
    ) {
        placement_delete_mq9_agent(
            client_pool,
            addrs,
            DeleteAgentRequest {
                tenant: tenant.to_string(),
                name: name.to_string(),
            },
        )
        .await
        .unwrap();
    }

    async fn list_agents(
        client_pool: &Arc<ClientPool>,
        addrs: &[String],
        tenant: &str,
    ) -> Vec<MQ9Agent> {
        let mut stream = placement_list_mq9_agent(
            client_pool,
            addrs,
            ListAgentRequest {
                tenant: tenant.to_string(),
            },
        )
        .await
        .unwrap();

        let mut agents = Vec::new();
        while let Ok(Some(reply)) = stream.message().await {
            agents.push(MQ9Agent::decode(&reply.agent).unwrap());
        }
        agents
    }

    #[tokio::test]
    async fn test_agent_register_list_unregister() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let tenant = unique_id();

        let card = make_agent_card(
            &format!("TestAgent-{}", unique_id()),
            "An agent for integration testing",
        );

        let before = list_agents(&client_pool, &addrs, &tenant).await;
        assert!(before.is_empty());

        register_agent(&client_pool, &addrs, &tenant, &card).await;

        // list is served from a node's local replica, which can briefly lag the
        // just-committed write; poll until it converges.
        let mut after = list_agents(&client_pool, &addrs, &tenant).await;
        for _ in 0..50 {
            if after.len() == 1 {
                break;
            }
            sleep(Duration::from_millis(100)).await;
            after = list_agents(&client_pool, &addrs, &tenant).await;
        }
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].name, card.name);
        assert_eq!(after[0].tenant, tenant);

        delete_agent(&client_pool, &addrs, &tenant, &card.name).await;

        let mut after_delete = list_agents(&client_pool, &addrs, &tenant).await;
        for _ in 0..50 {
            if after_delete.is_empty() {
                break;
            }
            sleep(Duration::from_millis(100)).await;
            after_delete = list_agents(&client_pool, &addrs, &tenant).await;
        }
        assert!(after_delete.is_empty());
    }

    #[tokio::test]
    async fn test_agent_search_vector_and_fts() {
        let client_pool = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let tenant = unique_id();

        let card = make_agent_card(
            &format!("PaymentAgent-{}", unique_id()),
            "Agent specialized in payment processing, invoices, and financial transactions",
        );

        register_agent(&client_pool, &addrs, &tenant, &card).await;

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let vector_reply = placement_search_mq9_agent(
            &client_pool,
            &addrs,
            SearchAgentRequest {
                tenant: tenant.clone(),
                semantic: "process a payment and generate invoice".to_string(),
                text: String::new(),
                limit: 5,
                offset: 0,
            },
        )
        .await
        .unwrap();
        assert!(!vector_reply.items.is_empty());
        assert_eq!(vector_reply.items[0].name, card.name);

        let fts_reply = placement_search_mq9_agent(
            &client_pool,
            &addrs,
            SearchAgentRequest {
                tenant: tenant.clone(),
                text: "payment invoices".to_string(),
                semantic: String::new(),
                limit: 5,
                offset: 0,
            },
        )
        .await
        .unwrap();
        assert!(!fts_reply.items.is_empty());
        assert_eq!(fts_reply.items[0].name, card.name);

        delete_agent(&client_pool, &addrs, &tenant, &card.name).await;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let vector_after = placement_search_mq9_agent(
            &client_pool,
            &addrs,
            SearchAgentRequest {
                tenant: tenant.clone(),
                semantic: "process a payment and generate invoice".to_string(),
                text: String::new(),
                limit: 5,
                offset: 0,
            },
        )
        .await
        .unwrap();
        assert!(vector_after.items.is_empty());

        let fts_after = placement_search_mq9_agent(
            &client_pool,
            &addrs,
            SearchAgentRequest {
                tenant: tenant.clone(),
                text: "payment invoices".to_string(),
                semantic: String::new(),
                limit: 5,
                offset: 0,
            },
        )
        .await
        .unwrap();
        assert!(fts_after.items.is_empty());
    }
}
