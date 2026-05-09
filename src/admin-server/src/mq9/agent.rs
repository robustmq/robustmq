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
use metadata_struct::mq9::agent::MQ9Agent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AgentListReq {
    pub tenant: Option<String>,
    pub name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
}

impl Queryable for MQ9Agent {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "name" => Some(self.name.clone()),
            "tenant" => Some(self.tenant.clone()),
            _ => None,
        }
    }
}

pub async fn agent_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<AgentListReq>,
) -> String {
    let nats_context = match &state.nats_context {
        Some(ctx) => ctx,
        None => return error_response("nats-broker is not running".to_string()),
    };

    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        None,
        None,
        None,
    );

    let agents: Vec<MQ9Agent> = nats_context
        .cache_manager
        .agent_info
        .iter()
        .filter(|e| {
            let agent = e.value();
            if let Some(tenant) = params.tenant.as_deref() {
                if agent.tenant != tenant {
                    return false;
                }
            }
            if let Some(keyword) = params.name.as_deref() {
                if !agent.name.contains(keyword) {
                    return false;
                }
            }
            true
        })
        .map(|e| e.value().clone())
        .collect();

    let sorted = apply_sorting(agents, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}
