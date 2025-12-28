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

use crate::core::error::StorageEngineError;
use crate::core::offset_index::SegmentOffsetIndex;
use crate::segment::file::SegmentFile;
use crate::segment::SegmentIdentity;
use broker_core::cache::BrokerCacheManager;
use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::storage::call::{list_segment, list_segment_meta, list_shard};
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::EngineSegment;
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

    // (shard_name, SegmentOffsetIndex)
    pub segment_offset_index: DashMap<String, SegmentOffsetIndex>,

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
        let segment_offset_index = DashMap::with_capacity(8);
        let leader_segments = DashMap::with_capacity(8);
        let segment_file_writer = DashMap::with_capacity(2);
        let is_next_segment = DashMap::with_capacity(2);
        StorageCacheManager {
            shards,
            segments,
            segment_metadatas,
            segment_offset_index,
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
        self.segments.remove(shard_name);
        self.segment_metadatas.remove(shard_name);
        self.segment_offset_index.remove(shard_name);

        for raw in self.leader_segments.iter() {
            if raw.shard_name == shard_name {
                self.leader_segments.remove(raw.key());
            }
        }

        for raw in self.segment_file_writer.iter() {
            if raw.shard_name == shard_name {
                self.segment_file_writer.remove(raw.key());
            }
        }

        self.is_next_segment.remove(shard_name);
    }

    // Segment
    pub fn set_segment(&self, segment: &EngineSegment) {
        let segment_list = self
            .segments
            .entry(segment.shard_name.clone())
            .or_insert(DashMap::with_capacity(8));
        segment_list.insert(segment.segment_seq, segment.clone());
    }

    pub fn delete_segment(&self, segment: &SegmentIdentity) {
        if let Some(list) = self.segments.get(&segment.shard_name) {
            list.remove(&segment.segment);
        }

        if let Some(list) = self.segment_metadatas.get(&segment.shard_name) {
            list.remove(&segment.segment);
        }

        self.leader_segments.remove(&segment.name());
        self.segment_file_writer.remove(&segment.name());

        if let Some(mut index) = self.segment_offset_index.get_mut(&segment.shard_name) {
            index.delete(segment.segment);
        }
    }

    pub fn get_segment(&self, segment: &SegmentIdentity) -> Option<EngineSegment> {
        if let Some(segment_list) = self.segments.get(&segment.shard_name) {
            if let Some(segment) = segment_list.get(&segment.segment) {
                return Some(segment.clone());
            }
        }
        None
    }

    pub fn get_segments_list_by_shard(&self, shard_name: &str) -> Vec<EngineSegment> {
        if let Some(segment_list) = self.segments.get(shard_name) {
            return segment_list.iter().map(|raw| raw.clone()).collect();
        }
        Vec::new()
    }

    pub fn get_segment_leader_nodes(&self, shard_name: &str) -> Vec<u64> {
        let segments = self.get_segments_list_by_shard(shard_name);
        let mut node_ids: Vec<u64> = segments.iter().map(|seg| seg.leader).collect();
        node_ids.sort_unstable();
        node_ids.dedup();
        node_ids
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

    // Segment Meta
    pub fn set_segment_meta(&self, segment: EngineSegmentMetadata) {
        let shard_name = segment.shard_name.clone();

        let data_list = self
            .segment_metadatas
            .entry(shard_name.clone())
            .or_insert(DashMap::with_capacity(8));

        data_list.insert(segment.segment_seq, segment.clone());

        let mut index = self.segment_offset_index.entry(shard_name).or_default();

        index.add(segment.segment_seq, segment.start_offset);
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

    // next segment
    pub fn add_next_segment(&self, shard: &str, segment: u32) {
        self.is_next_segment.insert(shard.to_string(), segment);
    }

    pub fn remove_next_segment(&self, shard: &str) {
        self.is_next_segment.remove(shard);
    }

    // Leader Segment
    pub fn remove_leader_segment(&self, segment_iden: &SegmentIdentity) {
        self.leader_segments.remove(&segment_iden.name());
    }

    pub fn add_leader_segment(&self, segment_iden: &SegmentIdentity) {
        self.leader_segments
            .insert(segment_iden.name(), segment_iden.clone());
    }

    // Segment Offset Index
    pub fn get_offset_index(&self, shard_name: &str) -> Option<SegmentOffsetIndex> {
        self.segment_offset_index
            .get(shard_name)
            .map(|entry| entry.clone())
    }

    pub fn sort_offset_index(&self, shard_name: &str) {
        if let Some(mut index) = self.segment_offset_index.get_mut(shard_name) {
            index.sort();
        }
    }
}

/// fetch node, shard, segment, segment meta from meta service and store them in cache
pub async fn load_metadata_cache(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), StorageEngineError> {
    let conf = broker_config();

    // load shard
    let request = ListShardRequest {
        ..Default::default()
    };
    let list = list_shard(client_pool, &conf.get_meta_service_addr(), request).await?;

    info!(
        "Load the shard cache, the number of shards is {}",
        list.shards.len()
    );

    for shard_bytes in list.shards {
        let shard = EngineShard::decode(&shard_bytes)?;
        cache_manager.set_shard(shard);
    }

    // load segment
    let request = ListSegmentRequest {
        segment: -1,
        ..Default::default()
    };
    let list = list_segment(client_pool, &conf.get_meta_service_addr(), request).await?;
    info!(
        "Load the segment cache, the number of segments is {}",
        list.segments.len()
    );
    for segment_bytes in list.segments {
        let segment = EngineSegment::decode(&segment_bytes)?;
        cache_manager.set_segment(&segment);
    }

    // load segment data
    let request = ListSegmentMetaRequest {
        segment: -1,
        ..Default::default()
    };
    let list = list_segment_meta(client_pool, &conf.get_meta_service_addr(), request).await?;
    info!(
        "Load the segment metadata cache, the number of segments is {}",
        list.segments.len()
    );
    for segment_bytes in list.segments {
        let meta = EngineSegmentMetadata::decode(&segment_bytes)?;
        cache_manager.set_segment_meta(meta);
    }

    for shard in cache_manager.shards.iter() {
        cache_manager.sort_offset_index(&shard.shard_name);
    }
    Ok(())
}
