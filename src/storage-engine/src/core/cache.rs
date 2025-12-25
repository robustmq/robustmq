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

use crate::segment::file::SegmentFile;
use crate::segment::SegmentIdentity;
use broker_core::cache::BrokerCacheManager;
use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::journal::call::{list_segment, list_segment_meta, list_shard};
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::{EngineSegment, SegmentStatus};
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use protocol::meta::meta_service_journal::{
    ListSegmentMetaRequest, ListSegmentRequest, ListShardRequest,
};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct StorageCacheManager {
    // broker cache
    pub broker_cache: Arc<BrokerCacheManager>,

    // (shard_name, JournalShard)
    pub shards: DashMap<String, EngineShard>,

    // (shard_name, (segment_no, JournalSegment))
    pub segments: DashMap<String, DashMap<u32, EngineSegment>>,

    // (shard_name, (segment_no, JournalSegmentMetadata))
    pub segment_metadatas: DashMap<String, DashMap<u32, EngineSegmentMetadata>>,

    // (segment_name, SegmentIdentity)
    pub leader_segments: DashMap<String, SegmentIdentity>,

    // (segment_name, SegmentFile)
    pub segment_file_writer: DashMap<String, SegmentFile>,

    // (shard_name, segment_seq)
    pub is_next_segment: DashMap<String, u32>,
}

impl StorageCacheManager {
    pub fn new(broker_cache: Arc<BrokerCacheManager>) -> Self {
        let shards = DashMap::with_capacity(8);
        let segments = DashMap::with_capacity(8);
        let segment_metadatas = DashMap::with_capacity(8);
        let leader_segments = DashMap::with_capacity(8);
        let segment_file_writer = DashMap::with_capacity(2);
        let is_next_segment = DashMap::with_capacity(2);
        StorageCacheManager {
            shards,
            segments,
            segment_metadatas,
            leader_segments,
            broker_cache,
            segment_file_writer,
            is_next_segment,
        }
    }

    // Shard
    pub fn set_shard(&self, shard: EngineShard) {
        self.shards.insert(shard.shard_name.clone(), shard);
    }

    pub fn delete_shard(&self, shard_name: &str) {
        self.shards.remove(shard_name);
    }

    pub fn get_active_segment(&self, shard_name: &str) -> Option<EngineSegment> {
        if let Some(shard) = self.shards.get(shard_name) {
            let segment_iden = SegmentIdentity {
                shard_name: shard_name.to_string(),
                segment: shard.active_segment_seq,
            };
            if let Some(segment) = self.get_segment(&segment_iden) {
                return Some(segment);
            }
        }

        None
    }

    // Segment
    pub fn set_segment(&self, segment: EngineSegment) {
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
                segment: segment.segment_seq,
            });
        }
    }

    pub fn delete_segment(&self, segment: &SegmentIdentity) {
        // delete segment
        if let Some(list) = self.segments.get(&segment.shard_name) {
            list.remove(&segment.segment);
        }

        // delete segment_metadatas
        if let Some(list) = self.segment_metadatas.get(&segment.shard_name) {
            list.remove(&segment.segment);
        }

        // delete leader segment
        self.remove_leader_segment(segment);
    }

    pub fn get_segment(&self, segment: &SegmentIdentity) -> Option<EngineSegment> {
        if let Some(sgement_list) = self.segments.get(&segment.shard_name) {
            if let Some(segment) = sgement_list.get(&segment.segment) {
                return Some(segment.clone());
            }
        }
        None
    }

    pub fn get_segments_list_by_shard(&self, shard_name: &str) -> Vec<EngineSegment> {
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
            if let Some(mut segment) = sgement_list.get_mut(&segment_iden.segment) {
                segment.status = status;
            }
        }
    }

    // Segment Meta
    pub fn set_segment_meta(&self, segment: EngineSegmentMetadata) {
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
    ) -> Option<EngineSegmentMetadata> {
        if let Some(list) = self.segment_metadatas.get(&segment_iden.shard_name) {
            if let Some(segment) = list.get(&segment_iden.segment) {
                return Some(segment.clone());
            }
        }
        None
    }

    // Segment File
    pub fn add_segment_file_write(
        &self,
        segment_iden: &SegmentIdentity,
        segment_file: SegmentFile,
    ) {
        self.segment_file_writer
            .insert(segment_iden.name(), segment_file);
    }

    pub fn remove_segment_file_write(&self, segment_iden: &SegmentIdentity) {
        self.segment_file_writer.remove(&segment_iden.name());
    }

    // next segment
    pub fn add_next_segment(&self, shard: &str, segment: u32) {
        self.is_next_segment.insert(shard.to_string(), segment);
    }

    pub fn remove_next_segment(&self, shard: &str) {
        self.is_next_segment.remove(shard);
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
}

/// fetch node, shard, segment, segment meta from meta service and store them in cache
pub async fn load_metadata_cache(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
) {
    let conf = broker_config();
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
                match EngineShard::decode(&shard_bytes) {
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
                match EngineSegment::decode(&segment_bytes) {
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
                match EngineSegmentMetadata::decode(&segment_bytes) {
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
}
