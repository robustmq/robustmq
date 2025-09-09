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
    request::mqtt::{CreateTopicRewriteReq, DeleteTopicRewriteReq, TopicListReq, TopicRewriteReq},
    response::{
        mqtt::{TopicListRow, TopicRewriteListRow},
        PageReplyData,
    },
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::{extract::State, Json};
use common_base::{
    http_response::{error_response, success_response},
    tools::now_mills,
};
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use mqtt_broker::storage::topic::TopicStorage;
use std::sync::Arc;

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
                topic_id: topic.topic_id.clone(),
                topic_name: topic.topic_name.clone(),
                is_contain_retain_message: topic.retain_message.is_none(),
            });
        }
    } else {
        for entry in state.mqtt_context.cache_manager.topic_info.iter() {
            let topic = entry.value();
            topics.push(TopicListRow {
                topic_id: topic.topic_id.clone(),
                topic_name: topic.topic_name.clone(),
                is_contain_retain_message: topic.retain_message.is_none(),
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
    Json(params): Json<CreateTopicRewriteReq>,
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

    success_response("success")
}

pub async fn topic_rewrite_delete(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<DeleteTopicRewriteReq>,
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
    success_response("success")
}
