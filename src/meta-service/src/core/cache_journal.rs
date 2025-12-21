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
use dashmap::DashMap;
use metadata_struct::storage::segment::{EngineSegment, SegmentStatus};
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;

impl CacheManager {
    // Shard
    pub fn get_shard(&self, shard_name: &str) -> Option<EngineShard> {
        let res = self.shard_list.get(shard_name)?;
        Some(res.clone())
    }

    pub fn set_shard(&self, shard: &EngineShard) {
        self.shard_list
            .insert(shard.shard_name.clone(), shard.clone());
    }

    pub fn remove_shard(&self, shard_name: &str) {
        self.shard_list.remove(shard_name);
        self.segment_list.remove(shard_name);
    }

    pub fn get_segment_list_by_shard(&self, shard_name: &str) -> Vec<EngineSegment> {
        let mut results = Vec::new();
        if let Some(segment_list) = self.segment_list.get(shard_name) {
            for raw in segment_list.iter() {
                results.push(raw.value().clone());
            }
        }
        results
    }

    pub fn get_segment_meta_list_by_shard(&self, shard_name: &str) -> Vec<EngineSegmentMetadata> {
        let mut results = Vec::new();
        if let Some(segment_list) = self.segment_meta_list.get(shard_name) {
            for raw in segment_list.iter() {
                results.push(raw.value().clone());
            }
        }
        results
    }

    pub fn next_segment_seq(&self, shard_name: &str) -> Option<u32> {
        if let Some(shard) = self.shard_list.get(shard_name) {
            return Some(shard.last_segment_seq + 1);
        }
        None
    }

    pub fn shard_idle_segment_num(&self, shard_name: &str) -> u32 {
        let mut num = 0;
        if let Some(segment_list) = self.segment_list.get(shard_name) {
            for segment in segment_list.iter() {
                if segment.status == SegmentStatus::Idle {
                    num += 1
                }
            }
        }
        num
    }

    pub fn get_segment(&self, shard_name: &str, segment_seq: u32) -> Option<EngineSegment> {
        if let Some(segment_list) = self.segment_list.get(shard_name) {
            let res = segment_list.get(&segment_seq)?;
            return Some(res.clone());
        }
        None
    }

    pub fn set_segment(&self, segment: &EngineSegment) {
        if let Some(shard_list) = self.segment_list.get(&segment.shard_name) {
            shard_list.insert(segment.segment_seq, segment.clone());
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(segment.segment_seq, segment.clone());
            self.segment_list.insert(segment.shard_name.clone(), data);
        }
    }

    pub fn remove_segment(&self, shard_name: &str, segment_seq: u32) {
        if let Some(segment_list) = self.segment_list.get(shard_name) {
            segment_list.remove(&segment_seq);
        }
    }

    pub fn get_segment_meta(
        &self,
        shard_name: &str,
        segment_seq: u32,
    ) -> Option<EngineSegmentMetadata> {
        if let Some(list) = self.segment_meta_list.get(shard_name) {
            let res = list.get(&segment_seq)?;
            return Some(res.clone());
        }
        None
    }

    pub fn set_segment_meta(&self, meta: &EngineSegmentMetadata) {
        if let Some(list) = self.segment_meta_list.get(&meta.shard_name) {
            list.insert(meta.segment_seq, meta.clone());
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(meta.segment_seq, meta.clone());
            self.segment_meta_list.insert(meta.shard_name.clone(), data);
        }
    }

    pub fn remove_segment_meta(&self, shard_name: &str, segment_seq: u32) {
        if let Some(list) = self.segment_meta_list.get(shard_name) {
            list.remove(&segment_seq);
        }
    }

    pub fn add_wait_delete_shard(&self, shard: &EngineShard) {
        self.wait_delete_shard_list
            .insert(shard.shard_name.to_string(), shard.clone());
    }

    pub fn remove_wait_delete_shard(&self, shard: &EngineShard) {
        self.wait_delete_shard_list
            .insert(shard.shard_name.to_string(), shard.clone());
    }

    pub fn add_wait_delete_segment(&self, segment: &EngineSegment) {
        self.wait_delete_segment_list.insert(
            self.segment_key(&segment.shard_name, segment.segment_seq),
            segment.clone(),
        );
    }

    pub fn remove_wait_delete_segment(&self, segment: &EngineSegment) {
        self.wait_delete_segment_list.insert(
            self.segment_key(&segment.shard_name, segment.segment_seq),
            segment.clone(),
        );
    }

    pub fn get_wait_delete_shard_list(&self) -> Vec<EngineShard> {
        let mut results = Vec::new();
        for raw in self.wait_delete_shard_list.iter() {
            results.push(raw.value().clone());
        }
        results
    }

    pub fn get_wait_delete_segment_list(&self) -> Vec<EngineSegment> {
        let mut results = Vec::new();
        for raw in self.wait_delete_segment_list.iter() {
            results.push(raw.value().clone());
        }
        results
    }

    fn segment_key(&self, shard_name: &str, segment_seq: u32) -> String {
        format!("{shard_name}_{segment_seq}")
    }
}
