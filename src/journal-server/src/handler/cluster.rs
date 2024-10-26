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

use protocol::journal_server::journal_engine::GetClusterMetadataNode;

use crate::core::cache::CacheManager;

#[derive(Clone)]
pub struct ClusterHandler {
    cache_manager: Arc<CacheManager>,
}

impl ClusterHandler {
    pub fn new(cache_manager: Arc<CacheManager>) -> ClusterHandler {
        ClusterHandler { cache_manager }
    }

    pub fn get_cluster_metadata(&self) -> Vec<GetClusterMetadataNode> {
        let mut result = Vec::new();
        for (node_id, node) in self.cache_manager.node_list.clone() {
            result.push(GetClusterMetadataNode {
                node_id,
                node_addr: node.node_inner_addr,
            });
        }
        result
    }
}
