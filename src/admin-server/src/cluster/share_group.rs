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
    tool::{
        query::{apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::extract::{Query, State};
use common_base::http_response::{error_response, success_response};
use metadata_struct::mqtt::share_group::{ShareGroup, ShareGroupMember};
use metadata_struct::nats::subscriber::NatsSubscriber;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ShareGroupListReq {
    pub tenant: Option<String>,
    pub group_name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShareGroupDetailReq {
    pub tenant: String,
    pub group_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueuePushThreadInfoView {
    pub total_pushed: u64,
    pub last_pull_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShareGroupDetailResp {
    pub group: ShareGroup,
    pub members: Vec<ShareGroupMember>,
    /// Active push subscribers in nats_core_queue_push for this queue group.
    pub push_subscribers: Vec<NatsSubscriber>,
    /// Runtime info of the push task thread; None if no thread is running.
    pub push_thread_info: Option<QueuePushThreadInfoView>,
}

impl Queryable for ShareGroup {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "tenant" => Some(self.tenant.clone()),
            "group_name" => Some(self.group_name.clone()),
            _ => None,
        }
    }
}

pub async fn share_group_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<ShareGroupListReq>,
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

    let cache = &state.broker_cache;
    let groups: Vec<ShareGroup> = cache
        .share_group_list
        .iter()
        .filter(|e| {
            let g = e.value();
            if let Some(t) = &params.tenant {
                if &g.tenant != t {
                    return false;
                }
            }
            if let Some(name) = &params.group_name {
                if !g.group_name.contains(name.as_str()) {
                    return false;
                }
            }
            true
        })
        .map(|e| e.value().clone())
        .collect();

    let sorted = apply_sorting(groups, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

pub async fn share_group_detail(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<ShareGroupDetailReq>,
) -> String {
    let cache = &state.broker_cache;
    let key = format!("{}/{}", params.tenant, params.group_name);

    let group = match cache.share_group_list.get(&key) {
        Some(g) => g.clone(),
        None => {
            return error_response(format!(
                "Share group '{}/{}' not found",
                params.tenant, params.group_name
            ));
        }
    };

    let members = cache
        .share_group_members
        .get(&key)
        .map(|m| m.clone())
        .unwrap_or_default();

    // queue_key format used in NatsSubscribeManager: "{tenant}#{group_name}#{subject}"
    // We don't know the subject here, so we match by prefix "{tenant}#{group_name}#"
    let queue_key_prefix = format!("{}#{}#", params.tenant, params.group_name);

    let (push_subscribers, push_thread_info) = if let Some(nats_ctx) = &state.nats_context {
        let sm = &nats_ctx.subscribe_manager;

        let subscribers: Vec<NatsSubscriber> = sm
            .nats_core_queue_push
            .iter()
            .filter(|e| e.key().starts_with(&queue_key_prefix))
            .flat_map(|e| {
                e.value()
                    .buckets_data_list
                    .iter()
                    .flat_map(|b| {
                        b.value()
                            .iter()
                            .map(|s| s.value().clone())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let thread_info = sm
            .nats_core_queue_push_thread
            .iter()
            .find(|e| e.key().starts_with(&queue_key_prefix))
            .map(|e| {
                let info = e.value();
                QueuePushThreadInfoView {
                    total_pushed: *info.total_pushed.lock().unwrap(),
                    last_pull_time: *info.last_pull_time.lock().unwrap(),
                }
            });

        (subscribers, thread_info)
    } else {
        (vec![], None)
    };

    success_response(ShareGroupDetailResp {
        group,
        members,
        push_subscribers,
        push_thread_info,
    })
}
