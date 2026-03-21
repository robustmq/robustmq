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

use crate::{
    state::HttpState,
    tool::extractor::ValidatedJson,
    tool::{
        query::{apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::extract::{Query, State};
use common_base::{
    http_response::{error_response, success_response},
    tools::now_millis,
};
use metadata_struct::mqtt::{message::MqttMessage, topic::Topic};
use metadata_struct::{
    mqtt::topic_rewrite_rule::MqttTopicRewriteRule, storage::shard::EngineShard,
};
use mqtt_broker::subscribe::manager::TopicSubscribeInfo;
use mqtt_broker::{core::error::MqttBrokerError, storage::topic::TopicStorage};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TopicListReq {
    pub tenant: Option<String>,
    pub topic_name: Option<String>,
    pub topic_type: Option<String>, // "all", "normal", "system"
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicDetailReq {
    pub tenant: String,
    pub topic_name: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct TopicDeleteRep {
    pub tenant: String,
    #[validate(length(
        min = 1,
        max = 256,
        message = "Topic name length must be between 1-256"
    ))]
    pub topic_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicRewriteReq {
    pub tenant: Option<String>,
    pub name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct CreateTopicRewriteReq {
    #[validate(length(min = 1, max = 256, message = "Name length must be between 1-256"))]
    pub name: String,

    pub desc: Option<String>,

    #[validate(length(min = 1, max = 128, message = "Tenant length must be between 1-128"))]
    pub tenant: String,

    #[validate(length(min = 1, max = 50, message = "Action length must be between 1-50"))]
    #[validate(custom(function = "validate_rewrite_action"))]
    pub action: String,

    #[validate(length(
        min = 1,
        max = 256,
        message = "Source topic length must be between 1-256"
    ))]
    pub source_topic: String,

    #[validate(length(
        min = 1,
        max = 256,
        message = "Dest topic length must be between 1-256"
    ))]
    pub dest_topic: String,

    #[validate(length(min = 1, max = 500, message = "Regex length must be between 1-500"))]
    pub regex: String,
}

fn validate_rewrite_action(action: &str) -> Result<(), validator::ValidationError> {
    match action {
        "All" | "Publish" | "Subscribe" => Ok(()),
        _ => {
            let mut err = validator::ValidationError::new("invalid_rewrite_action");
            err.message = Some(std::borrow::Cow::from(
                "Action must be All, Publish or Subscribe",
            ));
            Err(err)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct DeleteTopicRewriteReq {
    #[validate(length(min = 1, max = 128, message = "Tenant length must be between 1-128"))]
    pub tenant: String,

    #[validate(length(min = 1, max = 256, message = "Name length must be between 1-256"))]
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicDetailResp {
    pub topic_info: Topic,
    pub retain_message: Option<MqttMessage>,
    pub retain_message_at: Option<u64>,
    pub sub_list: HashSet<TopicSubscribeInfo>,
    pub storage_list: HashMap<u32, EngineShard>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicRewriteListRow {
    pub name: String,
    pub desc: String,
    pub tenant: String,
    pub action: String,
    pub source_topic: String,
    pub dest_topic: String,
    pub regex: String,
}

pub async fn topic_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<TopicListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        None,
        None,
        None,
    );

    let broker_cache = &state.mqtt_context.cache_manager.node_cache;
    let topics = collect_topics(
        broker_cache,
        params.tenant.as_deref(),
        params.topic_name.as_deref(),
        params.topic_type.as_deref(),
    );

    let sorted = apply_sorting(topics, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

/// Collect topics from cache, optionally filtered by tenant, topic_name (fuzzy), and topic_type.
/// topic_type: "system" (contains '$'), "normal" (no '$'), or "all" (default).
fn collect_topics(
    broker_cache: &broker_core::cache::NodeCacheManager,
    tenant: Option<&str>,
    topic_name: Option<&str>,
    topic_type: Option<&str>,
) -> Vec<Topic> {
    // Bulk listing with optional tenant filter
    let raw: Vec<Topic> = if let Some(t) = tenant {
        broker_cache.list_topics_by_tenant(t)
    } else {
        broker_cache
            .topic_list
            .iter()
            .flat_map(|e| {
                e.value()
                    .iter()
                    .map(|t| t.value().clone())
                    .collect::<Vec<_>>()
            })
            .collect()
    };

    raw.into_iter()
        .filter(|t| {
            if let Some(keyword) = topic_name {
                if !t.topic_name.contains(keyword) {
                    return false;
                }
            }
            match topic_type.unwrap_or("all") {
                "system" => t.topic_name.contains('$'),
                "normal" => !t.topic_name.contains('$'),
                _ => true,
            }
        })
        .collect()
}

impl Queryable for Topic {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "topic_name" => Some(self.topic_name.clone()),
            "tenant" => Some(self.tenant.clone()),
            _ => None,
        }
    }
}

pub async fn topic_detail(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<TopicDetailReq>,
) -> String {
    let result = match read_topic_detail(&state, &params).await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    success_response(result)
}

async fn read_topic_detail(
    state: &Arc<HttpState>,
    params: &TopicDetailReq,
) -> Result<TopicDetailResp, MqttBrokerError> {
    let topic = if let Some(topic) = state
        .mqtt_context
        .cache_manager
        .node_cache
        .get_topic_by_name(&params.tenant, &params.topic_name)
    {
        topic
    } else {
        return Err(MqttBrokerError::TopicDoesNotExist(
            params.topic_name.clone(),
        ));
    };

    let storage_list = state
        .mqtt_context
        .storage_driver_manager
        .list_storage_resource(&topic.tenant, &topic.topic_name)
        .await?;

    let sub_list = state
        .mqtt_context
        .subscribe_manager
        .topic_subscribes
        .get(&topic.tenant)
        .and_then(|t| t.get(&topic.topic_name).map(|v| v.clone()))
        .unwrap_or_default();
    let storage = TopicStorage::new(state.client_pool.clone());
    let (retain_message, retain_message_at) = storage
        .get_retain_message(&topic.tenant, &topic.topic_name)
        .await?;

    Ok(TopicDetailResp {
        topic_info: topic,
        retain_message,
        retain_message_at,
        sub_list,
        storage_list,
    })
}

pub async fn topic_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<TopicDeleteRep>,
) -> String {
    let topic_storage = TopicStorage::new(state.client_pool.clone());
    if let Err(e) = topic_storage
        .delete_topic(&params.tenant, &params.topic_name)
        .await
    {
        return error_response(e.to_string());
    }
    success_response("success")
}

pub async fn topic_rewrite_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<TopicRewriteReq>,
) -> String {
    let filter_tenant = params.tenant;
    let filter_name = params.name;
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        None,
        None,
        None,
    );

    let rewrite_rule_map = &state.mqtt_context.cache_manager.topic_rewrite_rule;
    let mut topic_rewrite_rules = Vec::new();

    for tenant_entry in rewrite_rule_map.iter() {
        let tenant = tenant_entry.key();
        if filter_tenant
            .as_deref()
            .map(|t| !tenant.contains(t))
            .unwrap_or(false)
        {
            continue;
        }
        for rule_entry in tenant_entry.value().iter() {
            let rule = rule_entry.value();
            if filter_name
                .as_deref()
                .map(|n| !rule.name.contains(n))
                .unwrap_or(false)
            {
                continue;
            }
            topic_rewrite_rules.push(TopicRewriteListRow {
                name: rule.name.clone(),
                desc: rule.desc.clone(),
                tenant: rule.tenant.clone(),
                action: rule.action.clone(),
                source_topic: rule.source_topic.clone(),
                dest_topic: rule.dest_topic.clone(),
                regex: rule.regex.clone(),
            });
        }
    }

    let sorted = apply_sorting(topic_rewrite_rules, &options);
    let pagination = apply_pagination(sorted, &options);
    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for TopicRewriteListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "name" => Some(self.name.clone()),
            "tenant" => Some(self.tenant.clone()),
            "source_topic" => Some(self.source_topic.clone()),
            "dest_topic" => Some(self.dest_topic.clone()),
            "action" => Some(self.action.clone()),
            _ => None,
        }
    }
}

pub async fn topic_rewrite_create(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<CreateTopicRewriteReq>,
) -> String {
    let rule = MqttTopicRewriteRule {
        name: params.name.clone(),
        desc: params.desc.clone().unwrap_or_default(),
        tenant: params.tenant.clone(),
        action: params.action.clone(),
        source_topic: params.source_topic.clone(),
        dest_topic: params.dest_topic.clone(),
        regex: params.regex.clone(),
        timestamp: now_millis(),
    };

    let topic_storage = TopicStorage::new(state.client_pool.clone());
    if let Err(e) = topic_storage.create_topic_rewrite_rule(rule.clone()).await {
        return error_response(e.to_string());
    }

    state
        .mqtt_context
        .cache_manager
        .set_re_calc_topic_rewrite(true)
        .await;
    success_response("success")
}

pub async fn topic_rewrite_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteTopicRewriteReq>,
) -> String {
    let topic_storage = TopicStorage::new(state.client_pool.clone());
    if let Err(e) = topic_storage
        .delete_topic_rewrite_rule(params.tenant.clone(), params.name.clone())
        .await
    {
        return error_response(e.to_string());
    }

    state
        .mqtt_context
        .cache_manager
        .set_re_calc_topic_rewrite(true)
        .await;
    success_response("success")
}
