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

use crate::core::error::MqttBrokerError;
use crate::core::tool::ResultMqttBrokerError;
use common_config::broker::broker_config;
use grpc_clients::meta::mqtt::call::{
    placement_create_topic_rewrite_rule, placement_delete_topic_rewrite_rule,
    placement_list_topic_rewrite_rule,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use protocol::meta::meta_service_mqtt::{
    CreateTopicRewriteRuleRequest, DeleteTopicRewriteRuleRequest, ListTopicRewriteRuleRequest,
};
use std::sync::Arc;

pub struct TopicRewriteStorage {
    client_pool: Arc<ClientPool>,
}

impl TopicRewriteStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        TopicRewriteStorage { client_pool }
    }

    pub async fn all_topic_rewrite_rule(
        &self,
    ) -> Result<Vec<MqttTopicRewriteRule>, MqttBrokerError> {
        self.topic_rewrite_rule_by_tenant("").await
    }

    pub async fn topic_rewrite_rule_by_tenant(
        &self,
        tenant: &str,
    ) -> Result<Vec<MqttTopicRewriteRule>, MqttBrokerError> {
        let config = broker_config();
        let request = ListTopicRewriteRuleRequest {
            tenant: tenant.to_string(),
        };
        let reply = placement_list_topic_rewrite_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        let mut results = Vec::with_capacity(8);
        for raw in reply.topic_rewrite_rules {
            let data = MqttTopicRewriteRule::decode(&raw)?;
            results.push(data);
        }
        Ok(results)
    }

    pub async fn create_topic_rewrite_rule(
        &self,
        req: MqttTopicRewriteRule,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = CreateTopicRewriteRuleRequest {
            name: req.name.clone(),
            desc: req.desc.clone(),
            tenant: req.tenant.clone(),
            action: req.action.clone(),
            source_topic: req.source_topic.clone(),
            dest_topic: req.dest_topic.clone(),
            regex: req.regex.clone(),
        };
        placement_create_topic_rewrite_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_topic_rewrite_rule(
        &self,
        tenant: String,
        name: String,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = DeleteTopicRewriteRuleRequest { tenant, name };
        placement_delete_topic_rewrite_rule(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        Ok(())
    }
}
