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

use crate::core::raft_node::RaftNode;

/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use super::server::HttpServerState;
use axum::extract::State;
use common_base::{http_response::success_response, metrics::dump_metrics};
use dashmap::DashMap;
use metadata_struct::placement::{broker_node::BrokerNode, cluster::ClusterInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct IndexResponse {
    pub local: RaftNode,
    pub votes: Vec<RaftNode>,
    pub members: Vec<RaftNode>,
    pub raft: RaftInfo,
}

#[derive(Serialize, Deserialize)]
pub struct RaftInfo {
    pub role: String,
    pub first_index: u64,
    pub last_index: u64,
    pub term: u64,
    pub vote: u64,
    pub commit: u64,
    pub voters: Vec<u64>,
    pub learners: Vec<u64>,
    pub voters_outgoing: Vec<u64>,
    pub learners_next: Vec<u64>,
    pub auto_leave: bool,
}

pub async fn index(State(state): State<HttpServerState>) -> String {
    let storage = state.raft_storage.read().unwrap();
    let hs = storage.hard_state();
    let cs = storage.conf_state();

    let raft_info = RaftInfo {
        role: format!("{:?}", state.placement_cache.get_current_raft_role()),
        first_index: storage.first_index(),
        last_index: storage.last_index(),
        term: hs.term,
        vote: hs.vote,
        commit: hs.commit,
        voters: cs.voters.to_vec(),
        learners: cs.learners.to_vec(),
        voters_outgoing: cs.voters_outgoing.to_vec(),
        learners_next: cs.learners_next.to_vec(),
        auto_leave: cs.auto_leave,
    };

    let resp = IndexResponse {
        local: state.placement_cache.get_raft_local_node().unwrap(),
        members: state.placement_cache.get_raft_members(),
        votes: state.placement_cache.get_raft_votes(),
        raft: raft_info,
    };

    return success_response(resp);
}

#[derive(Serialize, Deserialize)]
pub struct CacheInfo {
    pub cluster_list: DashMap<String, ClusterInfo>,
    pub node_list: DashMap<String, DashMap<u64, BrokerNode>>,
    pub node_heartbeat: DashMap<String, DashMap<u64, u64>>,
}
pub async fn caches(State(state): State<HttpServerState>) -> String {
    let cache: CacheInfo = CacheInfo {
        cluster_list: state.cluster_cache.cluster_list.clone(),
        node_list: state.cluster_cache.node_list.clone(),
        node_heartbeat: state.cluster_cache.node_heartbeat.clone(),
    };
    return success_response(cache);
}

pub async fn metrics() -> String {
    return dump_metrics();
}

pub async fn list_cluster(State(state): State<HttpServerState>) -> String {
    return success_response(state.cluster_cache.cluster_list.clone());
}

pub async fn list_node(State(state): State<HttpServerState>) -> String {
    return success_response(state.cluster_cache.node_list.clone());
}
