// Copyright 2023 RobustMQ Team
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

use crate::storage::journal::{segment::SegmentInfo, shard::ShardInfo};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct JournalCacheManager {
    pub shard_list: DashMap<String, ShardInfo>,
    pub segment_list: DashMap<String, SegmentInfo>,
}

impl JournalCacheManager {
    pub fn new() -> JournalCacheManager {
        return JournalCacheManager {
            shard_list: DashMap::with_capacity(8),
            segment_list: DashMap::with_capacity(256),
        };
    }

    pub fn add_shard(&self, shard: ShardInfo) {
        self.shard_list.insert(
            self.shard_key(shard.cluster_name.clone(), shard.shard_name.clone()),
            shard,
        );
    }

    pub fn remove_shard(&self, cluster_name: String, shard_name: String) {
        self.shard_list
            .remove(&self.shard_key(cluster_name, shard_name));
    }

    pub fn next_segment_seq(&self, cluster_name: &String, shard_name: &String) -> u64 {
        let key = self.shard_key(cluster_name.clone(), shard_name.clone());
        if let Some(shard) = self.shard_list.get(&key) {
            return shard.last_segment_seq + 1;
        }
        return 1;
    }
    
    #[allow(dead_code)]
    pub fn get_shard(&self, cluster_name: String, shard_name: String) -> Option<ShardInfo> {
        let key = self.shard_key(cluster_name, shard_name);
        if let Some(shard) = self.shard_list.get(&key) {
            return Some(shard.clone());
        }
        return None;
    }

    pub fn add_segment(&self, segment: SegmentInfo) {
        let key = self.segment_key(
            segment.cluster_name.clone(),
            segment.shard_name.clone(),
            segment.segment_seq,
        );

        self.segment_list.insert(key.clone(), segment.clone());
    }

    pub fn remove_segment(&self, cluster_name: String, shard_name: String, segment_seq: u64) {
        let key = self.segment_key(cluster_name.clone(), shard_name.clone(), segment_seq);
        self.segment_list.remove(&key);
    }

    fn shard_key(&self, cluster_name: String, shard_name: String) -> String {
        return format!("{}_{}", cluster_name, shard_name);
    }

    fn segment_key(&self, cluster_name: String, shard_name: String, segment_num: u64) -> String {
        return format!("{}_{}_{}", cluster_name, shard_name, segment_num);
    }
}
