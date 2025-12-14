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
use metadata_struct::journal::segment::{JournalSegment, SegmentStatus};
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::JournalShard;

impl CacheManager {
    // Shard
    pub fn get_shard(&self, namespace: &str, shard_name: &str) -> Option<JournalShard> {
        let key = self.shard_key(namespace, shard_name);
        let res = self.shard_list.get(&key)?;
        Some(res.clone())
    }

    pub fn set_shard(&self, shard: &JournalShard) {
        self.shard_list.insert(
            self.shard_key(&shard.namespace, &shard.shard_name),
            shard.clone(),
        );
    }

    pub fn remove_shard(&self, namespace: &str, shard_name: &str) {
        let key = self.shard_key(namespace, shard_name);
        self.shard_list.remove(&key);
        self.segment_list.remove(&key);
    }

    pub fn get_segment_list_by_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Vec<JournalSegment> {
        let key = self.shard_key(namespace, shard_name);
        let mut results = Vec::new();
        if let Some(segment_list) = self.segment_list.get(&key) {
            for raw in segment_list.iter() {
                results.push(raw.value().clone());
            }
        }
        results
    }

    pub fn get_segment_meta_list_by_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Vec<JournalSegmentMetadata> {
        let key = self.shard_key(namespace, shard_name);
        let mut results = Vec::new();
        if let Some(segment_list) = self.segment_meta_list.get(&key) {
            for raw in segment_list.iter() {
                results.push(raw.value().clone());
            }
        }
        results
    }

    pub fn next_segment_seq(&self, namespace: &str, shard_name: &str) -> Option<u32> {
        let key = self.shard_key(namespace, shard_name);
        if let Some(shard) = self.shard_list.get(&key) {
            return Some(shard.last_segment_seq + 1);
        }
        None
    }

    pub fn shard_idle_segment_num(&self, namespace: &str, shard_name: &str) -> u32 {
        let key = self.shard_key(namespace, shard_name);
        let mut num = 0;
        if let Some(segment_list) = self.segment_list.get(&key) {
            for segment in segment_list.iter() {
                if segment.status == SegmentStatus::Idle {
                    num += 1
                }
            }
        }
        num
    }

    pub fn get_segment(
        &self,
        namespace: &str,
        shard_name: &str,
        segment_seq: u32,
    ) -> Option<JournalSegment> {
        let key = self.shard_key(namespace, shard_name);
        if let Some(segment_list) = self.segment_list.get(&key) {
            let res = segment_list.get(&segment_seq)?;
            return Some(res.clone());
        }
        None
    }

    pub fn set_segment(&self, segment: &JournalSegment) {
        let key = self.shard_key(&segment.namespace, &segment.shard_name);
        if let Some(shard_list) = self.segment_list.get(&key) {
            shard_list.insert(segment.segment_seq, segment.clone());
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(segment.segment_seq, segment.clone());
            self.segment_list.insert(key.clone(), data);
        }
    }

    pub fn remove_segment(&self, namespace: &str, shard_name: &str, segment_seq: u32) {
        let key = self.shard_key(namespace, shard_name);
        if let Some(segment_list) = self.segment_list.get(&key) {
            segment_list.remove(&segment_seq);
        }
    }

    pub fn get_segment_meta(
        &self,
        namespace: &str,
        shard_name: &str,
        segment_seq: u32,
    ) -> Option<JournalSegmentMetadata> {
        let key = self.shard_key(namespace, shard_name);
        if let Some(list) = self.segment_meta_list.get(&key) {
            let res = list.get(&segment_seq)?;
            return Some(res.clone());
        }
        None
    }

    pub fn set_segment_meta(&self, meta: &JournalSegmentMetadata) {
        let key = self.shard_key(&meta.namespace, &meta.shard_name);
        if let Some(list) = self.segment_meta_list.get(&key) {
            list.insert(meta.segment_seq, meta.clone());
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(meta.segment_seq, meta.clone());
            self.segment_meta_list.insert(key.clone(), data);
        }
    }

    pub fn remove_segment_meta(&self, namespace: &str, shard_name: &str, segment_seq: u32) {
        let key = self.shard_key(namespace, shard_name);
        if let Some(list) = self.segment_meta_list.get(&key) {
            list.remove(&segment_seq);
        }
    }

    pub fn add_wait_delete_shard(&self, shard: &JournalShard) {
        self.wait_delete_shard_list.insert(
            self.shard_key(&shard.namespace, &shard.shard_name),
            shard.clone(),
        );
    }

    pub fn remove_wait_delete_shard(&self, shard: &JournalShard) {
        self.wait_delete_shard_list.insert(
            self.shard_key(&shard.namespace, &shard.shard_name),
            shard.clone(),
        );
    }

    pub fn add_wait_delete_segment(&self, segment: &JournalSegment) {
        self.wait_delete_segment_list.insert(
            self.segment_key(&segment.namespace, &segment.shard_name, segment.segment_seq),
            segment.clone(),
        );
    }

    pub fn remove_wait_delete_segment(&self, segment: &JournalSegment) {
        self.wait_delete_segment_list.insert(
            self.segment_key(&segment.namespace, &segment.shard_name, segment.segment_seq),
            segment.clone(),
        );
    }

    pub fn get_wait_delete_shard_list(&self) -> Vec<JournalShard> {
        let mut results = Vec::new();
        for raw in self.wait_delete_shard_list.iter() {
            results.push(raw.value().clone());
        }
        results
    }

    pub fn get_wait_delete_segment_list(&self) -> Vec<JournalSegment> {
        let mut results = Vec::new();
        for raw in self.wait_delete_segment_list.iter() {
            results.push(raw.value().clone());
        }
        results
    }

    fn shard_key(&self, namespace: &str, shard_name: &str) -> String {
        format!("{namespace}_{shard_name}")
    }

    fn segment_key(&self, namespace: &str, shard_name: &str, segment_seq: u32) -> String {
        format!("{namespace}_{shard_name}_{segment_seq}")
    }
}
