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
use grpc_clients::placement::inner::call::node_list;
use grpc_clients::placement::journal::call::{list_segment, list_segment_meta, list_shard};
use grpc_clients::pool::ClientPool;
use log::{debug, info};
use metadata_struct::journal::segment::{JournalSegment, SegmentStatus};
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::{shard_name_iden, JournalShard};
use metadata_struct::placement::node::BrokerNode;
use protocol::placement_center::placement_center_inner::NodeListRequest;
use protocol::placement_center::placement_center_journal::{
    ListSegmentMetaRequest, ListSegmentRequest, ListShardRequest,
};
use tokio::sync::broadcast;

use super::cluster::JournalEngineClusterConfig;
use crate::segment::write::SegmentWrite;
use crate::segment::SegmentIdentity;

#[derive(Clone)]
pub struct CacheManager {
    cluster: DashMap<String, JournalEngineClusterConfig>,
    node_list: DashMap<u64, BrokerNode>,
    shards: DashMap<String, JournalShard>,
    segments: DashMap<String, DashMap<u32, JournalSegment>>,
    segment_metadatas: DashMap<String, DashMap<u32, JournalSegmentMetadata>>,
    leader_segments: DashMap<String, SegmentIdentity>,
    segment_index_build_thread: DashMap<String, broadcast::Sender<bool>>,
    segment_writes: DashMap<String, SegmentWrite>,
}

impl CacheManager {
    pub fn new() -> Self {
        let cluster = DashMap::with_capacity(2);
        let node_list = DashMap::with_capacity(2);
        let shards = DashMap::with_capacity(8);
        let segments = DashMap::with_capacity(8);
        let segment_metadatas = DashMap::with_capacity(8);
        let leader_segments = DashMap::with_capacity(8);
        let segment_index_build_thread = DashMap::with_capacity(2);
        let segment_write = DashMap::with_capacity(2);
        CacheManager {
            cluster,
            node_list,
            shards,
            segments,
            segment_metadatas,
            leader_segments,
            segment_index_build_thread,
            segment_writes: segment_write,
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

    // Cluster
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

    // Shard
    pub fn set_shard(&self, shard: JournalShard) {
        let key = shard_name_iden(&shard.namespace, &shard.shard_name);
        self.shards.insert(key, shard);
    }

    pub fn get_shard(&self, namespace: &str, shard_name: &str) -> Option<JournalShard> {
        let key = shard_name_iden(namespace, shard_name);
        if let Some(shard) = self.shards.get(&key) {
            return Some(shard.clone());
        }
        None
    }

    pub fn delete_shard(&self, namespace: &str, shard_name: &str) {
        let key = shard_name_iden(namespace, shard_name);
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

    pub fn is_exist_shard(&self, namespace: &str, shard_name: &str) -> bool {
        let key = shard_name_iden(namespace, shard_name);
        self.shards.contains_key(&key) || self.segments.contains_key(&key)
    }

    pub fn get_active_segment(&self, namespace: &str, shard_name: &str) -> Option<JournalSegment> {
        let key = shard_name_iden(namespace, shard_name);
        if let Some(shard) = self.shards.get(&key) {
            let segment_iden = SegmentIdentity {
                namespace: namespace.to_string(),
                shard_name: shard_name.to_string(),
                segment_seq: shard.active_segment_seq,
            };
            if let Some(segment) = self.get_segment(&segment_iden) {
                return Some(segment);
            }
        }

        None
    }

    // Segment
    pub fn set_segment(&self, segment: JournalSegment) {
        let key = shard_name_iden(&segment.namespace, &segment.shard_name);

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
            self.add_leader_segment(&SegmentIdentity {
                namespace: segment.namespace,
                shard_name: segment.shard_name,
                segment_seq: segment.segment_seq,
            });
        }
    }

    pub fn delete_segment(&self, segment: &SegmentIdentity) {
        let key = shard_name_iden(&segment.namespace, &segment.shard_name);
        if let Some(list) = self.segments.get(&key) {
            list.remove(&segment.segment_seq);
        }

        if let Some(list) = self.segment_metadatas.get(&key) {
            list.remove(&segment.segment_seq);
        }

        self.remove_leader_segment(segment);

        if let Some(stop_send) = self.segment_index_build_thread.get(&key) {
            if let Err(e) = stop_send.send(true) {
                debug!("Trying to stop the index building thread for segment {} failed with error message:{}", segment.name(),e);
            }
        }

        if let Some(write) = self.segment_writes.get(&key) {
            if let Err(e) = write.stop_sender.send(true) {
                debug!("Trying to stop the segment write thread for segment {} failed with error message:{}", segment.name(),e);
            }
        }
    }

    pub fn get_segment(&self, segment: &SegmentIdentity) -> Option<JournalSegment> {
        let key = shard_name_iden(&segment.namespace, &segment.shard_name);
        if let Some(sgement_list) = self.segments.get(&key) {
            if let Some(segment) = sgement_list.get(&segment.segment_seq) {
                return Some(segment.clone());
            }
        }
        None
    }

    pub fn get_segments_list_by_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Vec<JournalSegment> {
        let mut results = Vec::new();
        if let Some(sgement_list) = self.segments.get(&shard_name_iden(namespace, shard_name)) {
            for raw in sgement_list.iter() {
                results.push(raw.value().clone());
            }
        }
        results
    }

    pub fn update_segment_status(&self, segment_iden: &SegmentIdentity, status: SegmentStatus) {
        if let Some(sgement_list) = self.segments.get(&shard_name_iden(
            &segment_iden.namespace,
            &segment_iden.shard_name,
        )) {
            if let Some(mut segment) = sgement_list.get_mut(&segment_iden.segment_seq) {
                segment.status = status;
            }
        }
    }

    // Segment Meta
    pub fn set_segment_meta(&self, segment: JournalSegmentMetadata) {
        let key = shard_name_iden(&segment.namespace, &segment.shard_name);
        if let Some(list) = self.segment_metadatas.get(&key) {
            list.insert(segment.segment_seq, segment);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(segment.segment_seq, segment);
            self.segment_metadatas.insert(key, data);
        }
    }

    pub fn delete_segment_meta(&self, segment: &JournalSegment) {
        let key = shard_name_iden(&segment.namespace, &segment.shard_name);
        if let Some(list) = self.segment_metadatas.get(&key) {
            list.remove(&segment.segment_seq);
        }
    }

    pub fn get_segment_meta(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Option<JournalSegmentMetadata> {
        let key = shard_name_iden(&segment_iden.namespace, &segment_iden.shard_name);
        if let Some(list) = self.segment_metadatas.get(&key) {
            if let Some(segment) = list.get(&segment_iden.segment_seq) {
                return Some(segment.clone());
            }
        }
        None
    }

    // Build Index Thread
    pub fn add_build_index_thread(
        &self,
        segment_iden: &SegmentIdentity,
        stop_send: broadcast::Sender<bool>,
    ) {
        self.segment_index_build_thread
            .insert(segment_iden.name(), stop_send);
    }

    pub fn remove_build_index_thread(&self, segment_iden: &SegmentIdentity) {
        let key = segment_iden.name();
        self.segment_index_build_thread.remove(&key);
    }

    pub fn contain_build_index_thread(&self, segment_iden: &SegmentIdentity) -> bool {
        self.segment_index_build_thread
            .contains_key(&segment_iden.name())
    }

    // Segment Write Thread
    pub fn add_segment_write_thread(
        &self,
        segment_iden: &SegmentIdentity,
        segment_write: SegmentWrite,
    ) {
        self.segment_writes
            .insert(segment_iden.name(), segment_write);
    }

    pub fn remove_segment_write_thread(&self, segment_iden: &SegmentIdentity) {
        self.segment_writes.remove(&segment_iden.name());
    }

    pub fn get_segment_write_thread(&self, segment_iden: &SegmentIdentity) -> Option<SegmentWrite> {
        if let Some(write) = self.segment_writes.get(&segment_iden.name()) {
            return Some(write.clone());
        }
        None
    }

    // Leader Segment
    pub fn get_leader_segment(&self) -> Vec<SegmentIdentity> {
        let mut results = Vec::new();
        for raw in self.leader_segments.iter() {
            results.push(raw.value().clone());
        }
        results
    }

    fn add_leader_segment(&self, segment_iden: &SegmentIdentity) {
        self.leader_segments
            .insert(segment_iden.name(), segment_iden.clone());
    }

    fn remove_leader_segment(&self, segment_iden: &SegmentIdentity) {
        self.leader_segments.remove(&segment_iden.name());
    }

    // Local cache
    pub fn is_allow_update_local_cache(&self) -> bool {
        if let Some(cluster) = self.cluster.get("local") {
            if (now_second() - cluster.last_update_local_cache_time) > 3 {
                return true;
            }
        }
        false
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
    match node_list(client_pool.clone(), &conf.placement_center, request).await {
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
    match list_shard(client_pool.clone(), &conf.placement_center, request).await {
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
    match list_segment(client_pool.clone(), &conf.placement_center, request).await {
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
    match list_segment_meta(client_pool.clone(), &conf.placement_center, request).await {
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
