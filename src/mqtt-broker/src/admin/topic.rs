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

use crate::handler::cache::CacheManager;
use crate::storage::topic::TopicStorage;
use common_base::{config::broker_mqtt::broker_mqtt_conf, tools::now_mills};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use protocol::broker_mqtt::broker_mqtt_admin::{
    CreateTopicRewriteRuleReply, CreateTopicRewriteRuleRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, ListTopicReply, ListTopicRequest, MqttTopic,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub fn list_topic_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<ListTopicRequest>,
) -> Result<Response<ListTopicReply>, Status> {
    let req = request.into_inner();
    let topic_query_result: Vec<MqttTopic> = match req.match_option {
        0 => cache_manager
            .get_topic_by_name(&req.topic_name)
            .into_iter()
            .take(10)
            .map(|entry| MqttTopic {
                topic_id: entry.topic_id.clone(),
                topic_name: entry.topic_name.clone(),
                cluster_name: entry.cluster_name.clone(),
                is_contain_retain_message: entry.retain_message.is_some(),
            })
            .collect(),
        option => cache_manager
            .topic_info
            .iter()
            .filter(|entry| match option {
                1 => entry.value().topic_name.starts_with(&req.topic_name),
                2 => entry.value().topic_name.contains(&req.topic_name),
                _ => false,
            })
            .take(10)
            .map(|entry| MqttTopic {
                topic_id: entry.value().topic_id.clone(),
                topic_name: entry.value().topic_name.clone(),
                cluster_name: entry.value().cluster_name.clone(),
                is_contain_retain_message: entry.value().retain_message.is_some(),
            })
            .collect(),
    };

    let reply = ListTopicReply {
        topics: topic_query_result,
    };

    Ok(Response::new(reply))
}

pub async fn delete_topic_rewrite_rule_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: Request<DeleteTopicRewriteRuleRequest>,
) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
    let req = request.into_inner();
    let topic_storage = TopicStorage::new(client_pool.clone());
    match topic_storage
        .delete_topic_rewrite_rule(req.action.clone(), req.source_topic.clone())
        .await
    {
        Ok(_) => {
            let config = broker_mqtt_conf();
            cache_manager.delete_topic_rewrite_rule(
                &config.cluster_name,
                &req.action,
                &req.source_topic,
            );
            Ok(Response::new(DeleteTopicRewriteRuleReply::default()))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn create_topic_rewrite_rule_by_req(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    request: Request<CreateTopicRewriteRuleRequest>,
) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();
    let topic_rewrite_rule = MqttTopicRewriteRule {
        cluster: config.cluster_name.clone(),
        action: req.action,
        source_topic: req.source_topic,
        dest_topic: req.dest_topic,
        regex: req.regex,
        timestamp: now_mills(),
    };
    let topic_storage = TopicStorage::new(client_pool.clone());
    match topic_storage
        .create_topic_rewrite_rule(topic_rewrite_rule.clone())
        .await
    {
        Ok(_) => {
            cache_manager.add_topic_rewrite_rule(topic_rewrite_rule);
            Ok(Response::new(CreateTopicRewriteRuleReply::default()))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}
