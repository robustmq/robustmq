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

use std::collections::{BTreeMap, BTreeSet};

use axum::extract::State;
use common_base::http_response::{error_response, success_response};
use openraft::error::Infallible;
use openraft::RaftMetrics;

use super::server::HttpServerState;
use crate::raftv2::raft_node::Node;
use crate::raftv2::typeconfig::TypeConfig;

pub async fn add_learner(State(state): State<HttpServerState>) -> String {
    let node_id = 3;
    let node = Node {
        rpc_addr: "127.0.0.0:7654".to_string(),
        node_id: 2,
    };

    match state
        .placement_center_storage
        .openraft_node
        .add_learner(node_id, node, true)
        .await
    {
        Ok(data) => success_response(data),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn change_membership(State(state): State<HttpServerState>) -> String {
    let mut body = BTreeSet::new();
    body.insert(3);
    match state
        .placement_center_storage
        .openraft_node
        .change_membership(body, true)
        .await
    {
        Ok(data) => success_response(data),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn init(State(state): State<HttpServerState>) -> String {
    let node_id = 3;
    let node = Node {
        rpc_addr: "127.0.0.0:7654".to_string(),
        node_id: 2,
    };

    let mut nodes = BTreeMap::new();
    nodes.insert(node_id, node);

    match state
        .placement_center_storage
        .openraft_node
        .initialize(nodes)
        .await
    {
        Ok(data) => success_response(data),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn raft_status(State(state): State<HttpServerState>) -> String {
    let metrics = state
        .placement_center_storage
        .openraft_node
        .metrics()
        .borrow()
        .clone();
    let res: Result<RaftMetrics<TypeConfig>, Infallible> = Ok(metrics);
    success_response(res)
}
