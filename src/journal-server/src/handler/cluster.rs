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

use metadata_struct::journal::node_extend::JournalNodeExtend;
use protocol::journal_server::journal_engine::GetClusterMetadataNode;

use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;

#[derive(Clone)]
pub struct ClusterHandler {
    cache_manager: Arc<CacheManager>,
}

impl ClusterHandler {
    pub fn new(cache_manager: Arc<CacheManager>) -> ClusterHandler {
        ClusterHandler { cache_manager }
    }

    pub fn get_cluster_metadata(&self) -> Result<Vec<GetClusterMetadataNode>, JournalServerError> {
        let mut result = Vec::new();
        for node in self.cache_manager.all_node() {
            let journal_extend = serde_json::from_str::<JournalNodeExtend>(&node.extend)?;
            result.push(GetClusterMetadataNode {
                node_id: node.node_id,
                tcp_addr: journal_extend.tcp_addr,
                tcps_addr: journal_extend.tcps_addr,
            });
        }
        Ok(result)
    }
}
