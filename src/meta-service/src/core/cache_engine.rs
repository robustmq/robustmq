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

use crate::core::cache::CacheManager;
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;

impl CacheManager {
    // Shard
    pub fn set_shard(&self, shard: &EngineShard) {
        self.shard_list
            .insert(shard.shard_name.clone(), shard.clone());
    }

    pub fn remove_shard(&self, shard_name: &str) {
        self.shard_list.remove(shard_name);
        self.segment_list.remove(shard_name);
    }

    // segment
    pub fn get_segment(&self, shard_name: &str, segment_seq: u32) -> Option<EngineSegment> {
        if let Some(segment_list) = self.segment_list.get(shard_name) {
            return Some(segment_list.get(&segment_seq)?.clone());
        }
        None
    }

    pub fn set_segment(&self, segment: &EngineSegment) {
        let list = self
            .segment_list
            .entry(segment.shard_name.clone())
            .or_insert(DashMap::with_capacity(8));
        list.insert(segment.segment_seq, segment.clone());
    }

    pub fn remove_segment(&self, shard_name: &str, segment_seq: u32) {
        if let Some(segment_list) = self.segment_list.get(shard_name) {
            segment_list.remove(&segment_seq);
        }
    }

    pub fn get_segment_list_by_shard(&self, shard_name: &str) -> Vec<EngineSegment> {
        if let Some(segment_list) = self.segment_list.get(shard_name) {
            return segment_list.iter().map(|raw| raw.clone()).collect();
        }
        Vec::new()
    }

    // segment meta
    pub fn get_segment_meta(
        &self,
        shard_name: &str,
        segment_seq: u32,
    ) -> Option<EngineSegmentMetadata> {
        if let Some(list) = self.segment_meta_list.get(shard_name) {
            return Some(list.get(&segment_seq)?.clone());
        }
        None
    }

    pub fn set_segment_meta(&self, meta: &EngineSegmentMetadata) {
        let list_data = self
            .segment_meta_list
            .entry(meta.shard_name.clone())
            .or_insert(DashMap::with_capacity(8));
        list_data.insert(meta.segment_seq, meta.clone());
    }

    pub fn remove_segment_meta(&self, shard_name: &str, segment_seq: u32) {
        if let Some(list) = self.segment_meta_list.get(shard_name) {
            list.remove(&segment_seq);
        }
    }

    pub fn get_segment_meta_list_by_shard(&self, shard_name: &str) -> Vec<EngineSegmentMetadata> {
        if let Some(segment_list) = self.segment_meta_list.get(shard_name) {
            return segment_list.iter().map(|raw| raw.clone()).collect();
        }
        Vec::new()
    }

    // await delete shard
    pub fn add_wait_delete_shard(&self, shard: &str) {
        self.wait_delete_shard_list
            .insert(shard.to_string(), now_second());
    }

    pub fn remove_wait_delete_shard(&self, shard: &str) {
        self.wait_delete_shard_list.remove(shard);
    }

    pub fn add_wait_delete_segment(&self, segment: &EngineSegment) {
        self.wait_delete_segment_list
            .insert(segment.name(), segment.clone());
    }

    pub fn remove_wait_delete_segment(&self, segment: &EngineSegment) {
        self.wait_delete_segment_list
            .insert(segment.name(), segment.clone());
    }

    pub fn get_wait_delete_shard_list(&self) -> Vec<String> {
        self.wait_delete_shard_list
            .iter()
            .map(|raw| raw.key().to_string())
            .collect()
    }

    pub fn get_wait_delete_segment_list(&self) -> Vec<EngineSegment> {
        self.wait_delete_segment_list
            .iter()
            .map(|raw| raw.clone())
            .collect()
    }
}
