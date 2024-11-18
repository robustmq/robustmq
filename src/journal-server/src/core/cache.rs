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
use common_base::tools::now_second;
use dashmap::DashMap;
use grpc_clients::placement::journal::call::{list_segment, list_segment_meta, list_shard};
use grpc_clients::placement::placement::call::node_list;
use grpc_clients::pool::ClientPool;
use log::{error, info};
use metadata_struct::journal::segment::{JournalSegment, SegmentStatus};
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::JournalShard;
use metadata_struct::placement::node::BrokerNode;
use protocol::journal_server::journal_inner::{
    JournalUpdateCacheActionType, JournalUpdateCacheResourceType,
};
use protocol::placement_center::placement_center_inner::NodeListRequest;
use protocol::placement_center::placement_center_journal::{
    ListSegmentMetaRequest, ListSegmentRequest, ListShardRequest,
};
use rocksdb_engine::RocksDBEngine;

use super::cluster::JournalEngineClusterConfig;
use crate::segment::manager::{create_local_segment, SegmentFileManager};
use crate::segment::SegmentIdentity;

#[derive(Clone)]
pub struct CacheManager {
    cluster: DashMap<String, JournalEngineClusterConfig>,
    node_list: DashMap<u64, BrokerNode>,
    shards: DashMap<String, JournalShard>,
    segments: DashMap<String, DashMap<u32, JournalSegment>>,
    segment_metadatas: DashMap<String, DashMap<u32, JournalSegmentMetadata>>,
    leader_segments: DashMap<String, SegmentIdentity>,
}

impl CacheManager {
    pub fn new() -> Self {
        let cluster = DashMap::with_capacity(2);
        let node_list = DashMap::with_capacity(2);
        let shards = DashMap::with_capacity(8);
        let segments = DashMap::with_capacity(8);
        let segment_metadatas = DashMap::with_capacity(8);
        let leader_segments = DashMap::with_capacity(8);
        CacheManager {
            cluster,
            node_list,
            shards,
            segments,
            segment_metadatas,
            leader_segments,
        }
    }

    pub fn add_node(&self, node: BrokerNode) {
        self.node_list.insert(node.node_id, node);
    }

    pub fn delete_node(&self, node_id: u64) {
        self.node_list.remove(&node_id);
    }

    pub fn all_node(&self) -> Vec<BrokerNode> {
        let mut results = Vec::new();
        for raw in self.node_list.iter() {
            results.push(raw.value().clone());
        }
        results
    }

    pub fn get_cluster(&self) -> JournalEngineClusterConfig {
        return self.cluster.get("local").unwrap().clone();
    }

    pub fn init_cluster(&self) {
        let cluster = JournalEngineClusterConfig::default();
        self.cluster.insert("local".to_string(), cluster);
    }

    pub fn update_local_cache_time(&self, time: u64) {
        if let Some(mut cluster) = self.cluster.get_mut("local") {
            cluster.last_update_local_cache_time = time;
        }
    }

    pub fn is_allow_update_local_cache(&self) -> bool {
        if let Some(cluster) = self.cluster.get("local") {
            if (now_second() - cluster.last_update_local_cache_time) > 3 {
                return true;
            }
        }
        false
    }

    pub fn set_shard(&self, shard: JournalShard) {
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
        self.segment_metadatas.remove(&key);
    }

    pub fn get_shards(&self) -> Vec<JournalShard> {
        let mut results = Vec::new();
        for raw in self.shards.iter() {
            results.push(raw.value().clone());
        }
        results
    }

    pub fn shard_is_exists(&self, namespace: &str, shard_name: &str) -> bool {
        let key = self.shard_key(namespace, shard_name);
        self.shards.contains_key(&key) || self.segments.contains_key(&key)
    }

    pub fn get_active_segment(&self, namespace: &str, shard_name: &str) -> Option<JournalSegment> {
        let key = self.shard_key(namespace, shard_name);
        if let Some(shard) = self.shards.get(&key) {
            if let Some(segment) = self.get_segment(namespace, shard_name, shard.active_segment_seq)
            {
                return Some(segment);
            }
        }

        None
    }

    pub fn set_segment(&self, segment: JournalSegment) {
        let key = self.shard_key(&segment.namespace, &segment.shard_name);
        if let Some(segment_list) = self.segments.get(&key) {
            segment_list.insert(segment.segment_seq, segment.clone());
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(segment.segment_seq, segment.clone());
            self.segments.insert(key, data);
        }

        // add to leader
        let conf = journal_server_conf();
        if segment.leader == conf.node_id {
            self.add_leader_segment(SegmentIdentity {
                namespace: segment.namespace,
                shard_name: segment.shard_name,
                segment_seq: segment.segment_seq,
            });
        }
    }

    pub fn delete_segment(&self, segment: &JournalSegment) {
        let key = self.shard_key(&segment.namespace, &segment.shard_name);
        if let Some(list) = self.segments.get(&key) {
            list.remove(&segment.segment_seq);
        }

        if let Some(list) = self.segment_metadatas.get(&key) {
            list.remove(&segment.segment_seq);
        }

        self.remove_leader_segment(&segment.namespace, &segment.shard_name, segment.segment_seq);
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

    pub fn get_segment_list_by_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Vec<JournalSegment> {
        let mut results = Vec::new();
        let key = self.shard_key(namespace, shard_name);
        if let Some(sgement_list) = self.segments.get(&key) {
            for raw in sgement_list.iter() {
                results.push(raw.value().clone());
            }
        }
        results
    }

    pub fn update_segment_status(
        &self,
        namespace: &str,
        shard_name: &str,
        segment_no: u32,
        status: SegmentStatus,
    ) {
        let key = self.shard_key(namespace, shard_name);
        if let Some(sgement_list) = self.segments.get(&key) {
            if let Some(mut segment) = sgement_list.get_mut(&segment_no) {
                segment.status = status;
            }
        }
    }

    pub fn set_segment_meta(&self, segment: JournalSegmentMetadata) {
        let key = self.shard_key(&segment.namespace, &segment.shard_name);
        if let Some(list) = self.segment_metadatas.get(&key) {
            list.insert(segment.segment_seq, segment);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(segment.segment_seq, segment);
            self.segment_metadatas.insert(key, data);
        }
    }

    pub fn delete_segment_meta(&self, segment: &JournalSegment) {
        let key = self.shard_key(&segment.namespace, &segment.shard_name);
        if let Some(list) = self.segment_metadatas.get(&key) {
            list.remove(&segment.segment_seq);
        }
    }

    pub fn get_segment_meta(
        &self,
        namespace: &str,
        shard_name: &str,
        segment_no: u32,
    ) -> Option<JournalSegmentMetadata> {
        let key = self.shard_key(namespace, shard_name);
        if let Some(list) = self.segment_metadatas.get(&key) {
            if let Some(segment) = list.get(&segment_no) {
                return Some(segment.clone());
            }
        }
        None
    }

    pub fn get_leader_segment(&self) -> Vec<SegmentIdentity> {
        let mut results = Vec::new();
        for raw in self.leader_segments.iter() {
            results.push(raw.value().clone());
        }
        results
    }

    fn add_leader_segment(&self, segment_iden: SegmentIdentity) {
        let key = self.sgement_key(
            &segment_iden.namespace,
            &segment_iden.shard_name,
            segment_iden.segment_seq,
        );
        self.leader_segments.insert(key, segment_iden);
    }

    fn remove_leader_segment(&self, namespace: &str, shard_name: &str, segment_seq: u32) {
        let key = self.sgement_key(namespace, shard_name, segment_seq);
        self.leader_segments.remove(&key);
    }

    fn sgement_key(&self, namespace: &str, shard_name: &str, segment_seq: u32) -> String {
        format!("{}_{}", namespace, shard_name)
    }

    fn shard_key(&self, namespace: &str, shard_name: &str) -> String {
        format!("{}_{}", namespace, shard_name)
    }
}

pub async fn update_cache(
    cache_manager: &Arc<CacheManager>,
    segment_file_manager: &Arc<SegmentFileManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    action_type: JournalUpdateCacheActionType,
    resource_type: JournalUpdateCacheResourceType,
    data: &str,
) {
    match resource_type {
        JournalUpdateCacheResourceType::JournalNode => parse_node(cache_manager, action_type, data),
        JournalUpdateCacheResourceType::Shard => parse_shard(cache_manager, action_type, data),
        JournalUpdateCacheResourceType::Segment => {
            parse_segment(
                cache_manager,
                segment_file_manager,
                rocksdb_engine_handler,
                action_type,
                data,
            )
            .await
        }
        JournalUpdateCacheResourceType::SegmentMeta => {
            parse_segment_meta(
                cache_manager,
                segment_file_manager,
                rocksdb_engine_handler,
                action_type,
                data,
            )
            .await
        }
    }
}

fn parse_node(
    cache_manager: &Arc<CacheManager>,
    action_type: JournalUpdateCacheActionType,
    data: &str,
) {
    match action_type {
        JournalUpdateCacheActionType::Set => match serde_json::from_str::<BrokerNode>(data) {
            Ok(node) => {
                info!("Update the cache, Set node, node id: {}", node.node_id);
                cache_manager.add_node(node);
            }
            Err(e) => {
                error!(
                    "Set node information failed to parse with error message :{},body:{}",
                    e, data,
                );
            }
        },

        JournalUpdateCacheActionType::Delete => match serde_json::from_str::<BrokerNode>(data) {
            Ok(node) => {
                info!("Update the cache, remove node, node id: {}", node.node_id);
                cache_manager.delete_node(node.node_id);
            }
            Err(e) => {
                error!(
                    "Remove node information failed to parse with error message :{},body:{}",
                    e, data,
                );
            }
        },
    }
}

fn parse_shard(
    cache_manager: &Arc<CacheManager>,
    action_type: JournalUpdateCacheActionType,
    data: &str,
) {
    match action_type {
        JournalUpdateCacheActionType::Set => match serde_json::from_str::<JournalShard>(data) {
            Ok(shard) => {
                info!(
                    "Update the cache, set shard, shard name: {}",
                    shard.shard_name
                );
                cache_manager.set_shard(shard);
            }
            Err(e) => {
                error!(
                    "set shard information failed to parse with error message :{},body:{}",
                    e, data,
                );
            }
        },

        _ => {
            error!(
                "UpdateCache updates Shard information, only supports Set operations, not {:?}",
                action_type
            );
        }
    }
}

async fn parse_segment(
    cache_manager: &Arc<CacheManager>,
    segment_file_manager: &Arc<SegmentFileManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    action_type: JournalUpdateCacheActionType,
    data: &str,
) {
    match action_type {
        JournalUpdateCacheActionType::Set => match serde_json::from_str::<JournalSegment>(data) {
            Ok(segment) => {
                info!(
                    "Segment cache update, action: set, segment:{}",
                    segment.name()
                );
                if cache_manager
                    .get_segment(&segment.namespace, &segment.shard_name, segment.segment_seq)
                    .is_some()
                {
                    return;
                }

                match create_local_segment(segment_file_manager, rocksdb_engine_handler, &segment)
                    .await
                {
                    Ok(()) => {
                        cache_manager.set_segment(segment);
                    }
                    Err(e) => {
                        error!("Error creating local Segment file, error message: {}", e);
                    }
                }
            }
            Err(e) => {
                error!(
                    "Set segment information failed to parse with error message :{},body:{}",
                    e, data,
                );
            }
        },
        _ => {
            error!(
                "UpdateCache updates Segment information, only supports Set operations, not {:?}",
                action_type
            );
        }
    }
}

async fn parse_segment_meta(
    cache_manager: &Arc<CacheManager>,
    segment_file_manager: &Arc<SegmentFileManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    action_type: JournalUpdateCacheActionType,
    data: &str,
) {
    match action_type {
        JournalUpdateCacheActionType::Set => {
            match serde_json::from_str::<JournalSegmentMetadata>(data) {
                Ok(segment) => {
                    info!(
                        "Update the cache, set segment meta, segment name:{}",
                        segment.name()
                    );
                    cache_manager.set_segment_meta(segment);
                }
                Err(e) => {
                    error!(
                        "Set segment meta information failed to parse with error message :{},body:{}",
                        e, data,
                    );
                }
            }
        }
        _ => {
            error!(
                "UpdateCache updates SegmentMeta information, only supports Set operations, not {:?}",
                action_type
            );
        }
    }
}

pub async fn load_metadata_cache(cache_manager: &Arc<CacheManager>, client_pool: &Arc<ClientPool>) {
    let conf = journal_server_conf();

    if !cache_manager.is_allow_update_local_cache() {
        return;
    }

    // load node
    let request = NodeListRequest {
        cluster_name: conf.cluster_name.clone(),
    };
    match node_list(client_pool.clone(), conf.placement_center.clone(), request).await {
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
    match list_shard(client_pool.clone(), conf.placement_center.clone(), request).await {
        Ok(list) => match serde_json::from_slice::<Vec<JournalShard>>(&list.shards) {
            Ok(data) => {
                info!(
                    "Load the shard cache, the number of shards is {}",
                    data.len()
                );
                for shard in data {
                    cache_manager.set_shard(shard);
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
        segment_no: -1,
        ..Default::default()
    };
    match list_segment(client_pool.clone(), conf.placement_center.clone(), request).await {
        Ok(list) => match serde_json::from_slice::<Vec<JournalSegment>>(&list.segments) {
            Ok(data) => {
                info!(
                    "Load the segment cache, the number of segments is {}",
                    data.len()
                );
                for shard in data {
                    cache_manager.set_segment(shard);
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
    // load segment data
    let request = ListSegmentMetaRequest {
        cluster_name: conf.cluster_name.clone(),
        segment_no: -1,
        ..Default::default()
    };
    match list_segment_meta(client_pool.clone(), conf.placement_center.clone(), request).await {
        Ok(list) => match serde_json::from_slice::<Vec<JournalSegmentMetadata>>(&list.segments) {
            Ok(data) => {
                info!(
                    "Load the segment metadata cache, the number of segments is {}",
                    data.len()
                );
                for meta in data {
                    cache_manager.set_segment_meta(meta);
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

    cache_manager.update_local_cache_time(now_second());
}
