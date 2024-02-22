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

use super::response::{success_response, IndexResponse, RaftInfo};
use crate::{
    cluster::Cluster,
    storage::{cluster_storage::ClusterStorage, raft_core::RaftRocksDBStorageCore},
};
use std::sync::{Arc, RwLock};

pub async fn controller_index(
    cluster: Arc<RwLock<Cluster>>,
    storage: Arc<RwLock<RaftRocksDBStorageCore>>,
) -> String {
    let cluster_read = cluster.read().unwrap();
    let storage = storage.read().unwrap();

    let hs = storage.hard_state();
    let cs = storage.conf_state();
    let uncommit_index = storage.uncommit_index();

    let raft_info = RaftInfo {
        role: format!("{:?}", cluster_read.raft_role),
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
        uncommit_index: uncommit_index,
    };

    let resp = IndexResponse {
        local: cluster_read.local.clone(),
        node_lists: cluster_read.peers.clone(),
        raft: raft_info,
    };

    return success_response(resp);
}

pub async fn cluster_info(cluster_storage: Arc<ClusterStorage>) -> String {
    return "".to_string();
}
