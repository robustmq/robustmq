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

use common_base::config::journal_server::journal_server_conf;
use dashmap::DashMap;
use grpc_clients::placement::journal::call::{list_segment, list_shard};
use grpc_clients::placement::placement::call::node_list;
use grpc_clients::poll::ClientPool;
use log::{error, info};
use metadata_struct::journal::segment::{JournalSegment, SegmentStatus};
use metadata_struct::journal::shard::JournalShard;
use metadata_struct::placement::node::BrokerNode;
use protocol::journal_server::journal_inner::{
    JournalUpdateCacheActionType, JournalUpdateCacheResourceType,
};
use protocol::placement_center::placement_center_inner::NodeListRequest;
use protocol::placement_center::placement_center_journal::{ListSegmentRequest, ListShardRequest};

use super::cluster::JournalEngineClusterConfig;
use super::shard::delete_shard;

#[derive(Clone)]
pub struct CacheManager {
    pub cluster: DashMap<String, JournalEngineClusterConfig>,
    pub node_list: DashMap<u64, BrokerNode>,
    shards: DashMap<String, JournalShard>,
    segments: DashMap<String, DashMap<u32, JournalSegment>>,
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

    pub fn add_node(&self, node: BrokerNode) {
        self.node_list.insert(node.node_id, node);
    }

    pub fn get_cluster(&self) -> JournalEngineClusterConfig {
        return self.cluster.get("local").unwrap().clone();
    }

    pub fn add_shard(&self, shard: JournalShard) {
        let key = self.shard_key(&shard.namespace, &shard.shard_name);
        self.shards.insert(key, shard);
    }

    pub fn get_shard(&self, namespace: &str, shard_name: &str) -> Option<JournalShard> {
        let key = self.shard_key(namespace, shard_name);
        if let Some(shard) = self.shards.get(&key) {
            return Some(shard.clone());
        }
        None
    }

    pub fn delete_shard(&self, namespace: &str, shard_name: &str) {
        let key = self.shard_key(namespace, shard_name);
        self.shards.remove(&key);
        self.segments.remove(&key);
    }

    pub fn get_active_segment(&self, namespace: &str, shard_name: &str) -> Option<JournalSegment> {
        let key = self.shard_key(namespace, shard_name);
        if let Some(shard) = self.shards.get(&key) {
            if let Some(segment) = self.get_segment(namespace, shard_name, shard.active_segment_seq)
            {
                if segment.status == SegmentStatus::Idle || segment.status == SegmentStatus::Write {
                    return Some(segment);
                }
            }
        }

        None
    }

    pub fn add_segment(&self, segment: JournalSegment) {
        let key = self.shard_key(&segment.namespace, &segment.shard_name);
        if let Some(segment_list) = self.segments.get(&key) {
            segment_list.insert(segment.segment_seq, segment);
        }
    }

    pub fn get_segment(
        &self,
        namespace: &str,
        shard_name: &str,
        segment_no: u32,
    ) -> Option<JournalSegment> {
        let key = self.shard_key(namespace, shard_name);
        if let Some(sgement_list) = self.segments.get(&key) {
            if let Some(segment) = sgement_list.get(&segment_no) {
                return Some(segment.clone());
            }
        }

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
            JournalUpdateCacheResourceType::Shard => self.parse_shard(action_type, data),
            JournalUpdateCacheResourceType::Segment => self.parse_segment(action_type, data),
        }
    }

    fn parse_node(&self, action_type: JournalUpdateCacheActionType, data: Vec<u8>) {
        match action_type {
            JournalUpdateCacheActionType::Add => {
                match serde_json::from_slice::<BrokerNode>(&data) {
                    Ok(node) => {
                        info!("Update the cache, add node, node id: {}", node.node_id);
                        self.node_list.insert(node.node_id, node);
                    }
                    Err(e) => {
                        error!(
                            "BrokerNode information failed to parse with error message :{}",
                            e
                        );
                    }
                }
            }

            JournalUpdateCacheActionType::Delete => match serde_json::from_slice::<u64>(&data) {
                Ok(node_id) => {
                    self.node_list.remove(&node_id);
                }
                Err(e) => {
                    error!(
                        "BrokerNode information failed to parse with error message :{}",
                        e
                    );
                }
            },
        }
    }

    fn parse_shard(&self, action_type: JournalUpdateCacheActionType, data: Vec<u8>) {
        match action_type {
            JournalUpdateCacheActionType::Add => {
                match serde_json::from_slice::<JournalShard>(&data) {
                    Ok(shard) => {
                        self.add_shard(shard);
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }

            JournalUpdateCacheActionType::Delete => {
                match serde_json::from_slice::<JournalShard>(&data) {
                    Ok(shard) => {
                        // Remove the shard and Segment information from the cache
                        self.delete_shard(&shard.namespace, &shard.shard_name);

                        // Delete the local segment file asynchronously
                        tokio::spawn(async move {
                            match delete_shard() {
                                Ok(()) => {}
                                Err(e) => {}
                            }
                        });
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }
    }

    fn parse_segment(&self, action_type: JournalUpdateCacheActionType, data: Vec<u8>) {
        match action_type {
            JournalUpdateCacheActionType::Add => {
                match serde_json::from_slice::<JournalSegment>(&data) {
                    Ok(shard) => {}
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }

            JournalUpdateCacheActionType::Delete => {
                match serde_json::from_slice::<JournalSegment>(&data) {
                    Ok(shard) => {}
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }
    }
}

pub async fn load_cache(client_poll: &Arc<ClientPool>, cache_manager: &Arc<CacheManager>) {
    let conf = journal_server_conf();
    // load node
    let request = NodeListRequest {
        cluster_name: conf.cluster_name.clone(),
    };
    match node_list(client_poll.clone(), conf.placement_center.clone(), request).await {
        Ok(list) => {
            info!(
                "Load the node cache, the number of nodes is {}",
                list.nodes.len()
            );
            for raw in list.nodes {
                let node = match serde_json::from_slice::<BrokerNode>(&raw) {
                    Ok(data) => data,
                    Err(e) => {
                        panic!("Failed to decode the BrokerNode information, {}", e);
                    }
                };
                cache_manager.add_node(node);
            }
        }
        Err(e) => {
            panic!(
                "Loading the node cache from the Placement Center failed, {}",
                e
            );
        }
    }

    // load shard
    let request = ListShardRequest {
        cluster_name: conf.cluster_name.clone(),
        ..Default::default()
    };
    match list_shard(client_poll.clone(), conf.placement_center.clone(), request).await {
        Ok(list) => match serde_json::from_slice::<Vec<JournalShard>>(&list.shards) {
            Ok(data) => {
                info!(
                    "Load the shard cache, the number of shards is {}",
                    data.len()
                );
                for shard in data {
                    cache_manager.add_shard(shard);
                }
            }
            Err(e) => {
                panic!("Failed to decode the JournalShard information, {}", e);
            }
        },
        Err(e) => {
            panic!(
                "Loading the shardcache from the Placement Center failed, {}",
                e
            );
        }
    }

    // load segment
    let request = ListSegmentRequest {
        cluster_name: conf.cluster_name.clone(),
        ..Default::default()
    };
    match list_segment(client_poll.clone(), conf.placement_center.clone(), request).await {
        Ok(list) => match serde_json::from_slice::<Vec<JournalSegment>>(&list.segments) {
            Ok(data) => {
                info!(
                    "Load the segment cache, the number of segments is {}",
                    data.len()
                );
                for shard in data {
                    cache_manager.add_segment(shard);
                }
            }
            Err(e) => {
                panic!("Failed to decode the JournalShard information, {}", e);
            }
        },
        Err(e) => {
            panic!(
                "Loading the shardcache from the Placement Center failed, {}",
                e
            );
        }
    }
    // load group
}
