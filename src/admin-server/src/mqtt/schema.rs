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

use std::sync::Arc;

use axum::extract::{Query, State};
use common_base::http_response::success_response;

use crate::{
    request::SchemaListReq,
    response::{PageReplyData, SchemaListRow},
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};

pub async fn schema_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<SchemaListReq>,
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

    let mut schemas = Vec::new();
    for schema in state.mqtt_context.schema_manager.get_all_schema() {
        schemas.push(SchemaListRow {
            name: schema.name.clone(),
            schema_type: schema.schema_type.to_string(),
            desc: schema.desc.clone(),
            schema: schema.schema.clone(),
        });
    }

    let filtered = apply_filters(schemas, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for SchemaListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "name" => Some(self.name.clone()),
            "schema_type" => Some(self.schema_type.clone()),
            _ => None,
        }
    }
}
