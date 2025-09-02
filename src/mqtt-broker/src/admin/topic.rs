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

use crate::admin::query::{apply_filters, apply_pagination, apply_sorting, Queryable};
use crate::handler::cache::MQTTCacheManager;
use crate::handler::error::MqttBrokerError;
use crate::storage::topic::TopicStorage;
use common_base::tools::now_mills;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use protocol::broker::broker_mqtt_admin::{
    CreateTopicRewriteRuleReply, CreateTopicRewriteRuleRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, ListRewriteTopicRuleReply, ListTopicReply, ListTopicRequest,
    MqttTopicRaw, MqttTopicRewriteRuleRaw,
};
use std::sync::Arc;

// List all topics by request
pub async fn list_topic_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    request: &ListTopicRequest,
) -> Result<ListTopicReply, MqttBrokerError> {
    let topics = extract_topic(cache_manager)?;

    if request.topic_name.as_deref().unwrap_or_default().is_empty() {
        let topic_count = topics.len();
        return Ok(ListTopicReply {
            topics,
            total_count: topic_count as u32,
        });
    }
    let filtered = apply_filters(topics, &request.options);
    let sorted = apply_sorting(filtered, &request.options);
    let pagination = apply_pagination(sorted, &request.options);

    Ok(ListTopicReply {
        topics: pagination.0,
        total_count: pagination.1 as u32,
    })
}

fn extract_topic(
    cache_manager: &Arc<MQTTCacheManager>,
) -> Result<Vec<MqttTopicRaw>, MqttBrokerError> {
    let mut topics = Vec::new();
    for entry in cache_manager.topic_info.iter() {
        let topic = entry.value();
        topics.push(MqttTopicRaw::from(topic.clone()));
    }
    Ok(topics)
}

// Delete a topic rewrite rule
pub async fn delete_topic_rewrite_rule_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<MQTTCacheManager>,
    request: &DeleteTopicRewriteRuleRequest,
) -> Result<DeleteTopicRewriteRuleReply, MqttBrokerError> {
    let topic_storage = TopicStorage::new(client_pool.clone());

    topic_storage
        .delete_topic_rewrite_rule(request.action.clone(), request.source_topic.clone())
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    let config = broker_config();
    cache_manager.delete_topic_rewrite_rule(
        &config.cluster_name,
        &request.action,
        &request.source_topic,
    );

    Ok(DeleteTopicRewriteRuleReply {})
}

// Create a topic rewrite rule
pub async fn create_topic_rewrite_rule_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<MQTTCacheManager>,
    request: &CreateTopicRewriteRuleRequest,
) -> Result<CreateTopicRewriteRuleReply, MqttBrokerError> {
    let config = broker_config();
    let rule = MqttTopicRewriteRule {
        cluster: config.cluster_name.clone(),
        action: request.action.clone(),
        source_topic: request.source_topic.clone(),
        dest_topic: request.dest_topic.clone(),
        regex: request.regex.clone(),
        timestamp: now_mills(),
    };

    let topic_storage = TopicStorage::new(client_pool.clone());
    topic_storage
        .create_topic_rewrite_rule(rule.clone())
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    cache_manager.add_topic_rewrite_rule(rule);

    Ok(CreateTopicRewriteRuleReply {})
}

pub async fn get_all_topic_rewrite_rule_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
) -> Result<ListRewriteTopicRuleReply, MqttBrokerError> {
    let mut topic_rewrite_rules = Vec::new();
    for entry in cache_manager.topic_rewrite_rule.iter() {
        let topic_rewrite_rule = entry.value();
        topic_rewrite_rules.push(MqttTopicRewriteRuleRaw::from(topic_rewrite_rule.clone()));
    }

    Ok(ListRewriteTopicRuleReply {
        rewrite_topic_rules: topic_rewrite_rules.clone(),
        total_count: topic_rewrite_rules.len() as u32,
    })
}

impl Queryable for MqttTopicRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "topic_id" => Some(self.topic_id.clone()),
            "cluster_name" => Some(self.cluster_name.clone()),
            "topic_name" => Some(self.topic_name.clone()),
            "is_contain_retain_message" => Some(self.is_contain_retain_message.to_string()),
            _ => None,
        }
    }
}
