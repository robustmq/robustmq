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

use super::cluster_config::JournalEngineClusterConfig;
use crate::index::build::IndexBuildThreadData;
use crate::segment::write::SegmentWrite;
use crate::segment::SegmentIdentity;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::common::call::node_list;
use grpc_clients::meta::journal::call::{list_segment, list_segment_meta, list_shard};
use grpc_clients::pool::ClientPool;
use metadata_struct::journal::segment::{JournalSegment, SegmentStatus};
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::JournalShard;
use metadata_struct::meta::node::BrokerNode;
use protocol::meta::meta_service_common::NodeListRequest;
use protocol::meta::meta_service_journal::{
    ListSegmentMetaRequest, ListSegmentRequest, ListShardRequest,
};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Clone)]
pub struct StorageCacheManager {
    // (shard_name, JournalShard)
    shards: DashMap<String, JournalShard>,

    // (shard_name, (segment_no, JournalSegment))
    segments: DashMap<String, DashMap<u32, JournalSegment>>,

    // (shard_name, (segment_no, JournalSegmentMetadata))
    segment_metadatas: DashMap<String, DashMap<u32, JournalSegmentMetadata>>,

    // (segment_name, SegmentIdentity)
    leader_segments: DashMap<String, SegmentIdentity>,

    // (segment_name, IndexBuildThreadData)
    segment_index_build_thread: DashMap<String, IndexBuildThreadData>,

    // (segment_name, SegmentWrite)
    segment_writes: DashMap<String, SegmentWrite>,
}

impl Default for StorageCacheManager {
    fn default() -> Self {
        Self::new()
    }
}
impl StorageCacheManager {
    pub fn new() -> Self {
        let shards = DashMap::with_capacity(8);
        let segments = DashMap::with_capacity(8);
        let segment_metadatas = DashMap::with_capacity(8);
        let leader_segments = DashMap::with_capacity(8);
        let segment_index_build_thread = DashMap::with_capacity(2);
        let segment_write = DashMap::with_capacity(2);
        StorageCacheManager {
            shards,
            segments,
            segment_metadatas,
            leader_segments,
            segment_index_build_thread,
            segment_writes: segment_write,
        }
    }

    pub fn is_allow_update_local_cache(&self) -> bool {
        if (now_second() - self.get_cluster().last_update_local_cache_time) > 3 {
            return true;
        }

        false
    }

    // Shard
    pub fn set_shard(&self, shard: JournalShard) {
        self.shards.insert(shard.shard_name.clone(), shard);
    }

    pub fn get_shard(&self, shard_name: &str) -> Option<JournalShard> {
        if let Some(shard) = self.shards.get(shard_name) {
            return Some(shard.clone());
        }
        None
    }

    pub fn delete_shard(&self, shard_name: &str) {
        self.shards.remove(shard_name);
    }

    pub fn get_shards(&self) -> Vec<JournalShard> {
        self.shards.iter().map(|raw| raw.value().clone()).collect()
    }

    pub fn get_shards_by_namespace(&self, namespace: &str) -> Vec<JournalShard> {
        let mut results = Vec::new();
        let prefix = format!("{namespace},");
        for raw in self.shards.iter() {
            if raw.key().starts_with(&prefix) {
                results.push(raw.value().clone());
            }
        }
        results
    }

    pub fn get_active_segment(&self, shard_name: &str) -> Option<JournalSegment> {
        if let Some(shard) = self.shards.get(shard_name) {
            let segment_iden = SegmentIdentity {
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
        if let Some(segment_list) = self.segments.get(&segment.shard_name) {
            segment_list.insert(segment.segment_seq, segment.clone());
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(segment.segment_seq, segment.clone());
            self.segments.insert(segment.shard_name.clone(), data);
        }

        // add to leader
        let conf = broker_config();
        if segment.leader == conf.broker_id {
            self.add_leader_segment(&SegmentIdentity {
                shard_name: segment.shard_name,
                segment_seq: segment.segment_seq,
            });
        }
    }

    pub fn delete_segment(&self, segment: &SegmentIdentity) {
        // delete segment
        if let Some(list) = self.segments.get(&segment.shard_name) {
            list.remove(&segment.segment_seq);
        }

        // delete segment_metadatas
        if let Some(list) = self.segment_metadatas.get(&segment.shard_name) {
            list.remove(&segment.segment_seq);
        }

        // delete leader segment
        self.remove_leader_segment(segment);

        // delete index build thread by segment
        if let Some(data) = self.segment_index_build_thread.get(&segment.shard_name) {
            if let Err(e) = data.stop_send.send(true) {
                error!("Trying to stop the index building thread for segment {} failed with error message:{}", segment.name(),e);
            }
        }

        // delete write thread by segment
        if let Some(write) = self.segment_writes.get(&segment.shard_name) {
            if let Err(e) = write.stop_sender.send(true) {
                error!("Trying to stop the segment write thread for segment {} failed with error message:{}", segment.name(),e);
            }
        }
    }

    pub fn get_segment(&self, segment: &SegmentIdentity) -> Option<JournalSegment> {
        if let Some(sgement_list) = self.segments.get(&segment.shard_name) {
            if let Some(segment) = sgement_list.get(&segment.segment_seq) {
                return Some(segment.clone());
            }
        }
        None
    }

    pub fn get_segments_list_by_shard(&self, shard_name: &str) -> Vec<JournalSegment> {
        let mut results = Vec::new();
        if let Some(sgement_list) = self.segments.get(shard_name) {
            for raw in sgement_list.iter() {
                results.push(raw.value().clone());
            }
        }
        results
    }

    pub fn update_segment_status(&self, segment_iden: &SegmentIdentity, status: SegmentStatus) {
        if let Some(sgement_list) = self.segments.get(&segment_iden.shard_name) {
            if let Some(mut segment) = sgement_list.get_mut(&segment_iden.segment_seq) {
                segment.status = status;
            }
        }
    }

    // Segment Meta
    pub fn set_segment_meta(&self, segment: JournalSegmentMetadata) {
        if let Some(list) = self.segment_metadatas.get(&segment.shard_name) {
            list.insert(segment.segment_seq, segment);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(segment.segment_seq, segment.clone());
            self.segment_metadatas
                .insert(segment.shard_name.clone(), data);
        }
    }

    pub fn get_segment_meta(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Option<JournalSegmentMetadata> {
        if let Some(list) = self.segment_metadatas.get(&segment_iden.shard_name) {
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
        index_build_thread_data: IndexBuildThreadData,
    ) {
        self.segment_index_build_thread
            .insert(segment_iden.name(), index_build_thread_data);
    }

    pub fn remove_build_index_thread(&self, segment_iden: &SegmentIdentity) {
        if let Some((_, data)) = self.segment_index_build_thread.remove(&segment_iden.name()) {
            if let Err(e) = data.stop_send.send(true) {
                error!("Trying to stop the index building thread for segment {} failed with error message:{}", segment_iden.name(),e);
            }
        }
    }

    pub fn contain_build_index_thread(&self, segment_iden: &SegmentIdentity) -> bool {
        self.segment_index_build_thread
            .contains_key(&segment_iden.name())
    }

    pub fn stop_all_build_index_thread(&self) {
        for raw in self.segment_index_build_thread.iter() {
            if let Err(e) = raw.value().stop_send.send(true) {
                error!("Trying to stop the index building thread for segment {} failed with error message:{}", raw.key(),e);
            }
        }
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

    // get start time
    pub fn get_start_time(&self) -> u64 {
        self.start_time
    }
}

/// fetch node, shard, segment, segment meta from meta service and store them in cache
pub async fn load_metadata_cache(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
) {
    let conf = broker_config();

    if !cache_manager.is_allow_update_local_cache() {
        return;
    }

    // load node
    let request = NodeListRequest {};
    match node_list(client_pool, &conf.get_meta_service_addr(), request).await {
        Ok(list) => {
            info!(
                "Load the node cache, the number of nodes is {}",
                list.nodes.len()
            );
            for raw in list.nodes {
                let node = match BrokerNode::decode(&raw) {
                    Ok(data) => data,
                    Err(e) => {
                        panic!("Failed to decode the BrokerNode information, {e}");
                    }
                };
                cache_manager.add_node(node);
            }
        }
        Err(e) => {
            panic!("Loading the node cache from the Meta Service failed, {e}");
        }
    }

    // load shard
    let request = ListShardRequest {
        ..Default::default()
    };
    match list_shard(client_pool, &conf.get_meta_service_addr(), request).await {
        Ok(list) => {
            info!(
                "Load the shard cache, the number of shards is {}",
                list.shards.len()
            );
            for shard_bytes in list.shards {
                match JournalShard::decode(&shard_bytes) {
                    Ok(shard) => cache_manager.set_shard(shard),
                    Err(e) => {
                        panic!("Failed to decode the JournalShard information, {e}");
                    }
                }
            }
        }
        Err(e) => {
            panic!("Loading the shardcache from the Meta Service failed, {e}");
        }
    }

    // load segment
    let request = ListSegmentRequest {
        segment_no: -1,
        ..Default::default()
    };
    match list_segment(client_pool, &conf.get_meta_service_addr(), request).await {
        Ok(list) => {
            info!(
                "Load the segment cache, the number of segments is {}",
                list.segments.len()
            );
            for segment_bytes in list.segments {
                match JournalSegment::decode(&segment_bytes) {
                    Ok(segment) => cache_manager.set_segment(segment),
                    Err(e) => {
                        panic!("Failed to decode the JournalSegment information, {e}");
                    }
                }
            }
        }
        Err(e) => {
            panic!("Loading the segment cache from the Meta Service failed, {e}");
        }
    }
    // load segment data
    let request = ListSegmentMetaRequest {
        segment_no: -1,
        ..Default::default()
    };
    match list_segment_meta(client_pool, &conf.get_meta_service_addr(), request).await {
        Ok(list) => {
            info!(
                "Load the segment metadata cache, the number of segments is {}",
                list.segments.len()
            );
            for segment_bytes in list.segments {
                match JournalSegmentMetadata::decode(&segment_bytes) {
                    Ok(meta) => cache_manager.set_segment_meta(meta),
                    Err(e) => {
                        panic!("Failed to decode the JournalSegmentMetadata information, {e}");
                    }
                }
            }
        }
        Err(e) => {
            panic!("Loading the segment metadata cache from the Meta Service failed, {e}");
        }
    }

    cache_manager.update_local_cache_time(now_second());
}
