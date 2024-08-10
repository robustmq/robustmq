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


use crate::storage::{keys::key_shard, rocksdb::RocksDBEngine};

use super::segment::{SegmentInfo, SegmentStorage};
use common_base::log::error_meta;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ShardInfo {
    pub shard_uid: String,
    pub cluster_name: String,
    pub shard_name: String,
    pub replica: u32,
    pub last_segment_seq: u64,
    pub segments: Vec<u64>,
    pub create_time: u128,
}

pub struct ShardStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShardStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ShardStorage {
            rocksdb_engine_handler,
        }
    }

    // save shard info
    pub fn save(&self, shard_info: ShardInfo) {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let shard_key = key_shard(
            &shard_info.cluster_name.clone(),
            shard_info.shard_name.clone(),
        );
        match self
            .rocksdb_engine_handler
            .write(cf, &shard_key, &shard_info)
        {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    // get shard info
    pub fn get(&self, cluster_name: String, shard_name: String) -> Option<ShardInfo> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let shard_key: String = key_shard(&cluster_name, shard_name);
        match self
            .rocksdb_engine_handler
            .read::<ShardInfo>(cf, &shard_key)
        {
            Ok(ci) => {
                return ci;
            }
            Err(_) => {}
        }
        return None;
    }

    // delete shard info
    pub fn delete(&self, cluster_name: String, shard_name: String) {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let shard_key = key_shard(&cluster_name, shard_name);
        match self.rocksdb_engine_handler.delete(cf, &shard_key) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_all_shard(&self, _: &String) -> Vec<String> {

        return Vec::new();
    }

    #[allow(dead_code)]
    pub fn shard_list(&self, cluster_name: String) -> Vec<ShardInfo> {
        let all_shard = self.get_all_shard(&cluster_name);
        let mut result = Vec::new();
        for shard in all_shard {
            if let Some(shard_info) = self.get(cluster_name.clone(), shard) {
                result.push(shard_info);
            }
        }
        return result;
    }

    pub fn add_segment(&self, cluster_name: String, shard_name: String, segment_seq: u64) {
        if let Some(mut shard) = self.get(cluster_name, shard_name) {
            shard.segments.push(segment_seq);
            shard.last_segment_seq = segment_seq;
            self.save(shard);
        }
    }

    pub fn delete_segment(&self, cluster_name: String, shard_name: String, segment_seq: u64) {
        if let Some(mut shard) = self.get(cluster_name, shard_name) {
            match shard.segments.binary_search(&segment_seq) {
                Ok(index) => {
                    shard.segments.remove(index);
                    self.save(shard);
                }
                Err(_) => {}
            }
        }
    }

    pub fn segment_list(&self, cluster_name: String, shard_name: String) -> Vec<SegmentInfo> {
        let mut result = Vec::new();
        let segment_handler = SegmentStorage::new(self.rocksdb_engine_handler.clone());
        if let Some(shard) = self.get(cluster_name.clone(), shard_name.clone()) {
            for seq in shard.segments {
                if let Some(segment) =
                    segment_handler.get_segment(cluster_name.clone(), shard_name.clone(), seq)
                {
                    result.push(segment);
                }
            }
        }
        return result;
    }
}
