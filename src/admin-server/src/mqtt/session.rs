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

use crate::{request::SessionListReq, state::HttpState};
use axum::extract::{Query, State};
use common_base::http_response::success_response;
use std::sync::Arc;

pub async fn session_list(
    State(_state): State<Arc<HttpState>>,
    Query(params): Query<SessionListReq>,
) -> String {
    success_response(params.client_id)
}
