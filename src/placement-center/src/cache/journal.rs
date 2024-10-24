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

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::storage::journal::segment::SegmentInfo;
use crate::storage::journal::shard::ShardInfo;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct JournalCacheManager {
    pub shard_list: DashMap<String, ShardInfo>,
    pub segment_list: DashMap<String, DashMap<u32, SegmentInfo>>,
}

impl JournalCacheManager {
    pub fn new() -> JournalCacheManager {
        JournalCacheManager {
            shard_list: DashMap::with_capacity(8),
            segment_list: DashMap::with_capacity(256),
        }
    }

    pub fn get_shard(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Option<ShardInfo> {
        let key = self.shard_key(cluster_name, namespace, shard_name);
        let res = self.shard_list.get(&key)?;
        Some(res.clone())
    }

    pub fn add_shard(&self, shard: &ShardInfo) {
        self.shard_list.insert(
            self.shard_key(&shard.cluster_name, &shard.namespace, &shard.shard_name),
            shard.clone(),
        );
    }

    pub fn remove_shard(&self, cluster_name: &str, namespace: &str, shard_name: &str) {
        let key = self.shard_key(cluster_name, namespace, shard_name);
        self.shard_list.remove(&key);
        self.segment_list.remove(&key);
    }

    pub fn next_segment_seq(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Option<u32> {
        let key = self.shard_key(cluster_name, namespace, shard_name);
        if let Some(shard) = self.shard_list.get(&key) {
            return Some(shard.last_segment_seq + 1);
        }
        None
    }

    pub fn add_segment(&self, segment: SegmentInfo) {
        let key = self.shard_key(
            &segment.cluster_name,
            &segment.namespace,
            &segment.shard_name,
        );
        if let Some(shard_list) = self.segment_list.get(&key) {
            shard_list.insert(segment.segment_seq, segment.clone());
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(segment.segment_seq, segment.clone());
            self.segment_list.insert(key.clone(), data);
        }

        if let Some(mut shard) = self.shard_list.get_mut(&key) {
            shard.last_segment_seq = segment.segment_seq;
        }
    }

    pub fn get_segment(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        segment_seq: u32,
    ) -> Option<SegmentInfo> {
        let key = self.shard_key(cluster_name, namespace, shard_name);
        if let Some(segment_list) = self.segment_list.get(&key) {
            let res = segment_list.get(&segment_seq)?;
            return Some(res.clone());
        }
        None
    }

    pub fn remove_segment(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        segment_seq: u32,
    ) {
        let key = self.shard_key(cluster_name, namespace, shard_name);
        if let Some(segment_list) = self.segment_list.get(&key) {
            if let Some(mut shard) = self.shard_list.get_mut(&key) {
                shard.start_segment_seq = segment_seq + 1;
            }
            segment_list.remove(&segment_seq);
        }
    }

    fn shard_key(&self, cluster_name: &str, namespace: &str, shard_name: &str) -> String {
        format!("{}_{}_{}", cluster_name, namespace, shard_name)
    }
}
