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
use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::storage::topic::TopicStorage;
use common_base::{config::broker_mqtt::broker_mqtt_conf, tools::now_mills};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use protocol::broker_mqtt::broker_mqtt_admin::{
    CreateTopicRewriteRuleRequest, DeleteTopicRewriteRuleRequest, ListTopicRequest, MqttTopicRaw,
};
use std::sync::Arc;
use tonic::Request;

// List all topics by request
pub async fn list_topic_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<ListTopicRequest>,
) -> Result<(Vec<MqttTopicRaw>, usize), MqttBrokerError> {
    let req = request.into_inner();
    let topics = extract_topic(cache_manager)?;

    if req.topic_name.as_deref().unwrap_or_default().is_empty() {
        let topic_count = topics.len();
        return Ok((topics, topic_count));
    }
    let filtered = apply_filters(topics, &req.options);
    let sorted = apply_sorting(filtered, &req.options);
    let pagination = apply_pagination(sorted, &req.options);
    Ok(pagination)
}

fn extract_topic(cache_manager: &Arc<CacheManager>) -> Result<Vec<MqttTopicRaw>, MqttBrokerError> {
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
    cache_manager: &Arc<CacheManager>,
    request: Request<DeleteTopicRewriteRuleRequest>,
) -> Result<(), MqttBrokerError> {
    let req = request.into_inner();
    let topic_storage = TopicStorage::new(client_pool.clone());

    topic_storage
        .delete_topic_rewrite_rule(req.action.clone(), req.source_topic.clone())
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    let config = broker_mqtt_conf();
    cache_manager.delete_topic_rewrite_rule(&config.cluster_name, &req.action, &req.source_topic);

    Ok(())
}

// Create a topic rewrite rule
pub async fn create_topic_rewrite_rule_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: Request<CreateTopicRewriteRuleRequest>,
) -> Result<(), MqttBrokerError> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();
    let rule = MqttTopicRewriteRule {
        cluster: config.cluster_name.clone(),
        action: req.action,
        source_topic: req.source_topic,
        dest_topic: req.dest_topic,
        regex: req.regex,
        timestamp: now_mills(),
    };

    let topic_storage = TopicStorage::new(client_pool.clone());
    topic_storage
        .create_topic_rewrite_rule(rule.clone())
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    cache_manager.add_topic_rewrite_rule(rule);

    Ok(())
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
