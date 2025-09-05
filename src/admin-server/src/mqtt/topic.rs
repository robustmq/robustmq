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
    request::TopicListReq,
    response::{PageReplyData, TopicListRow},
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::extract::{Query, State};
use common_base::http_response::success_response;
use std::sync::Arc;

pub async fn topic_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<TopicListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.page_num,
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
