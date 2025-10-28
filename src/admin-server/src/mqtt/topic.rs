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
    extractor::ValidatedJson,
    state::HttpState,
    tool::{
        query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::{extract::State, Json};
use common_base::{
    http_response::{error_response, success_response},
    tools::now_mills,
};
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use metadata_struct::mqtt::{message::MqttMessage, topic::MQTTTopic};
use mqtt_broker::storage::topic::TopicStorage;
use mqtt_broker::subscribe::manager::TopicSubscribeInfo;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicListReq {
    pub topic_name: Option<String>,
    pub topic_type: Option<String>, // "all", "normal", "system"
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicDetailReq {
    pub topic_name: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct TopicDeleteRep {
    #[validate(length(
        min = 1,
        max = 256,
        message = "Topic name length must be between 1-256"
    ))]
    pub topic_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicRewriteReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct CreateTopicRewriteReq {
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
    #[validate(length(min = 1, max = 50, message = "Action length must be between 1-50"))]
    #[validate(custom(function = "validate_rewrite_action"))]
    pub action: String,

    #[validate(length(
        min = 1,
        max = 256,
        message = "Source topic length must be between 1-256"
    ))]
    pub source_topic: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicListRow {
    pub topic_name: String,
    pub create_time: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicDetailResp {
    pub topic_info: MQTTTopic,
    pub retain_message: Option<MqttMessage>,
    pub retain_message_at: Option<u64>,
    pub sub_list: Vec<TopicSubscribeInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicRewriteListRow {
    pub source_topic: String,
    pub dest_topic: String,
    pub regex: String,
    pub action: String,
}

pub async fn topic_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<TopicListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );

    let mut topics = Vec::new();

    if let Some(tp) = params.topic_name.clone() {
        if let Some(topic) = state.mqtt_context.cache_manager.get_topic_by_name(&tp) {
            topics.push(TopicListRow {
                topic_name: topic.topic_name.clone(),
                create_time: topic.create_time,
            });
        }
    } else {
        let topic_type = params.topic_type.as_deref().unwrap_or("all");
        for entry in state.mqtt_context.cache_manager.topic_info.iter() {
            let topic = entry.value();
            let allow = if topic_type == "system" {
                entry.topic_name.contains("$")
            } else if topic_type == "normal" {
                !entry.topic_name.contains("$")
            } else {
                true
            };

            if !allow {
                continue;
            }

            topics.push(TopicListRow {
                topic_name: topic.topic_name.clone(),
                create_time: topic.create_time,
            });
        }
    }

    let filtered = apply_filters(topics, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for TopicListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "topic_name" => Some(self.topic_name.clone()),
            _ => None,
        }
    }
}

pub async fn topic_detail(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<TopicDetailReq>,
) -> String {
    let topic = if let Some(topic) = state
        .mqtt_context
        .cache_manager
        .get_topic_by_name(&params.topic_name)
    {
        topic
    } else {
        return error_response("Topic does not exist.".to_string());
    };

    let sub_list = state
        .mqtt_context
        .subscribe_manager
        .get_topic_subscribe_list(&topic.topic_name);

    let storage = TopicStorage::new(state.client_pool.clone());
    let (retain_message, retain_message_at) =
        match storage.get_retain_message(&topic.topic_name).await {
            Ok((retain_message, retain_message_at)) => (retain_message, retain_message_at),
            Err(e) => {
                return error_response(e.to_string());
            }
        };

    success_response(TopicDetailResp {
        topic_info: topic,
        retain_message,
        retain_message_at,
        sub_list,
    })
}

pub async fn topic_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<TopicDeleteRep>,
) -> String {
    let topic_storage = TopicStorage::new(state.client_pool.clone());
    if let Err(e) = topic_storage.delete_topic(&params.topic_name).await {
        return error_response(e.to_string());
    }
    success_response("success")
}

pub async fn topic_rewrite_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<TopicRewriteReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );

    let mut topic_rewrite_rules = Vec::new();
    for entry in state.mqtt_context.cache_manager.topic_rewrite_rule.iter() {
        let topic_rewrite_rule = entry.value();
        topic_rewrite_rules.push(TopicRewriteListRow {
            source_topic: topic_rewrite_rule.source_topic.clone(),
            dest_topic: topic_rewrite_rule.dest_topic.clone(),
            action: topic_rewrite_rule.action.clone(),
            regex: topic_rewrite_rule.regex.clone(),
        });
    }

    let filtered = apply_filters(topic_rewrite_rules, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);
    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for TopicRewriteListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
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
        cluster: state.broker_cache.cluster_name.clone(),
        action: params.action.clone(),
        source_topic: params.source_topic.clone(),
        dest_topic: params.dest_topic.clone(),
        regex: params.regex.clone(),
        timestamp: now_mills(),
    };

    let topic_storage = TopicStorage::new(state.client_pool.clone());
    if let Err(e) = topic_storage.create_topic_rewrite_rule(rule.clone()).await {
        return error_response(e.to_string());
    }

    state
        .mqtt_context
        .cache_manager
        .add_topic_rewrite_rule(rule);

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
        .delete_topic_rewrite_rule(params.action.clone(), params.source_topic.clone())
        .await
    {
        return error_response(e.to_string());
    }
    state.mqtt_context.cache_manager.delete_topic_rewrite_rule(
        &state.broker_cache.cluster_name,
        &params.action,
        &params.source_topic,
    );
    state
        .mqtt_context
        .cache_manager
        .set_re_calc_topic_rewrite(true)
        .await;
    success_response("success")
}
