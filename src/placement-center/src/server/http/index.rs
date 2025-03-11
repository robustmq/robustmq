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

use axum::extract::State;
use common_base::metrics::dump_metrics;

use crate::server::http::server::HttpServerState;

pub async fn metrics() -> String {
    dump_metrics()
}

pub async fn raft_metrics(state: State<HttpServerState>) -> String {
    let metrics = state
        .placement_center_storage
        .openraft_node
        .metrics()
        .borrow()
        .clone();
    serde_json::to_string(&metrics).unwrap()
}
