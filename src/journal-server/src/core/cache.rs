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

use dashmap::DashMap;
use grpc_clients::placement::placement::call::node_list;
use grpc_clients::poll::ClientPool;
use log::error;
use metadata_struct::journal::segment::JournalSegment;
use metadata_struct::journal::shard::JournalShard;
use metadata_struct::placement::node::BrokerNode;
use protocol::journal_server::journal_inner::{
    JournalUpdateCacheActionType, JournalUpdateCacheResourceType,
};
use protocol::placement_center::placement_center_inner::NodeListRequest;

use super::cluster::JournalEngineClusterConfig;

#[derive(Clone)]
pub struct CacheManager {
    pub cluster: DashMap<String, JournalEngineClusterConfig>,
    pub node_list: DashMap<u64, BrokerNode>,
    shards: DashMap<String, JournalShard>,
    segments: DashMap<String, DashMap<u64, JournalSegment>>,
}

impl CacheManager {
    pub fn new() -> Self {
        let cluster = DashMap::with_capacity(2);
        let node_list = DashMap::with_capacity(2);
        let shards = DashMap::with_capacity(8);
        let segments = DashMap::with_capacity(8);
        CacheManager {
            cluster,
            node_list,
            shards,
            segments,
        }
    }

    pub async fn load_cache(
        &self,
        client_poll: Arc<ClientPool>,
        addrs: Vec<String>,
        cluster_name: String,
    ) {
        // load node
        let request = NodeListRequest { cluster_name };

        match node_list(client_poll, addrs, request).await {
            Ok(list) => {
                for raw in list.nodes {
                    let node = match serde_json::from_slice::<BrokerNode>(&raw) {
                        Ok(data) => data,
                        Err(e) => {
                            panic!("Failed to decode the BrokerNode information, {}", e);
                        }
                    };
                    self.node_list.insert(node.node_id, node);
                }
            }
            Err(e) => {
                panic!("Loading the cache from the Placement Center failed, {}", e);
            }
        }

        // load shard

        // load segment

        // load group
    }

    pub fn get_cluster(&self) -> JournalEngineClusterConfig {
        return self.cluster.get("local").unwrap().clone();
    }
    pub fn get_shard(&self, namespace: &str, shard_name: &str) -> Option<JournalShard> {
        let key = self.shard_key(namespace, shard_name);
        if let Some(shard) = self.shards.get(&key) {
            return Some(shard.clone());
        }
        None
    }

    pub fn get_active_segment(&self, namespace: &str, shard_name: &str) -> Option<JournalSegment> {
        let key = self.shard_key(namespace, shard_name);
        if let Some(shard) = self.shards.get(&key) {}

        None
    }

    pub fn shard_exists(&self, namespace: &str, shard_name: &str) -> bool {
        let key = self.shard_key(namespace, shard_name);
        self.shards.contains_key(&key)
    }

    fn shard_key(&self, namespace: &str, shard_name: &str) -> String {
        format!("{}_{}", namespace, shard_name)
    }

    pub fn update_cache(
        &self,
        action_type: JournalUpdateCacheActionType,
        resource_type: JournalUpdateCacheResourceType,
        data: Vec<u8>,
    ) {
        match resource_type {
            JournalUpdateCacheResourceType::JournalNode => self.parse_node(action_type, data),
            JournalUpdateCacheResourceType::Shard => {}
            JournalUpdateCacheResourceType::Segment => {}
        }
    }

    fn parse_node(&self, action_type: JournalUpdateCacheActionType, data: Vec<u8>) {
        match action_type {
            JournalUpdateCacheActionType::Add => {
                match serde_json::from_slice::<BrokerNode>(&data) {
                    Ok(node) => {
                        self.node_list.insert(node.node_id, node);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }

            JournalUpdateCacheActionType::Delete => match serde_json::from_slice::<u64>(&data) {
                Ok(node_id) => {
                    self.node_list.remove(&node_id);
                }
                Err(e) => {
                    error!("{}", e);
                }
            },
        }
    }
}
