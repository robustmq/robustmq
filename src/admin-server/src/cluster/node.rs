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

use crate::{state::HttpState, tool::extractor::ValidatedJson};
use axum::extract::State;
use broker_core::cluster::ClusterStorage;
use common_base::http_response::{error_response, success_response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct LeaveClusterNodeReq {
    #[validate(range(min = 1, message = "node_id must be >= 1"))]
    pub node_id: u64,
    /// Remove the node even if it is still registered (alive). Default false:
    /// the request is rejected for a live node to avoid orphaning it.
    #[serde(default)]
    pub force: bool,
}

/// Permanently remove a node from the Raft cluster (scale-in).
///
/// This is an operational action, not something that happens on a normal
/// restart. Stop the target node's process first, then call this. By default a
/// still-registered (alive) node is refused (pass `force: true` to override).
/// After removal, wipe the node's data directory before it can rejoin as a new
/// node. Quorum safety is enforced on the meta side.
pub async fn node_leave(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<LeaveClusterNodeReq>,
) -> String {
    if !params.force && state.broker_cache.node_lists.contains_key(&params.node_id) {
        return error_response(format!(
            "Node {} is still registered (alive). Stop its process first, or pass force=true. \
             Removing a live node would orphan it.",
            params.node_id
        ));
    }

    let storage = ClusterStorage::new(state.client_pool.clone());
    match storage.leave_cluster(params.node_id).await {
        Ok(_) => success_response(format!(
            "Node {} removed from the cluster. Wipe its data directory before rejoining as a new node.",
            params.node_id
        )),
        Err(e) => error_response(e.to_string()),
    }
}
