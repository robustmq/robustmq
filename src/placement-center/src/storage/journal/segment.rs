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

use crate::storage::{
    engine::{
        engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
        engine_save_by_cluster,
    },
    keys::{key_segment, key_segment_cluster_prefix, key_segment_shard_prefix},
    rocksdb::RocksDBEngine,
};
use common_base::error::common::CommonError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct SegmentInfo {
    pub cluster_name: String,
    pub shard_name: String,
    pub segment_seq: u64,
    pub replicas: Vec<Replica>,
    pub replica_leader: u32,
    pub status: SegmentStatus,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Replica {
    pub replica_seq: u64,
    pub node_id: u64,
    pub fold: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum SegmentStatus {
    #[default]
    Idle,
    Write,
    PrepareSealUp,
    SealUp,
}

pub struct SegmentStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SegmentStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        SegmentStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, segment: SegmentInfo) -> Result<(), CommonError> {
        let shard_key = key_segment(
            &segment.cluster_name.clone(),
            &segment.shard_name.clone(),
            segment.segment_seq,
        );
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), shard_key, segment);
    }

    #[allow(dead_code)]
    pub fn get(
        &self,
        cluster_name: &String,
        shard_name: &String,
        segment_seq: u64,
    ) -> Result<Option<SegmentInfo>, CommonError> {
        let shard_key: String = key_segment(cluster_name, shard_name, segment_seq);

        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), shard_key) {
            Ok(Some(data)) => match serde_json::from_slice::<SegmentInfo>(&data.data) {
                Ok(segment) => {
                    return Ok(Some(segment));
                }
                Err(e) => {
                    return Err(e.into());
                }
            },
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => Err(e),
        }
    }

    pub fn list_by_cluster(
        &self,
        cluster_name: &String,
    ) -> Result<Vec<SegmentInfo>, CommonError> {
        let prefix_key = key_segment_cluster_prefix(cluster_name);
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<SegmentInfo>(&raw.data) {
                        Ok(topic) => {
                            results.push(topic);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                return Ok(results);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn list_by_shard(
        &self,
        cluster_name: &String,
        shard_name: &String,
    ) -> Result<Vec<SegmentInfo>, CommonError> {
        let prefix_key = key_segment_shard_prefix(cluster_name, shard_name);
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<SegmentInfo>(&raw.data) {
                        Ok(topic) => {
                            results.push(topic);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                return Ok(results);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn delete(
        &self,
        cluster_name: &String,
        shard_name: &String,
        segment_seq: u64,
    ) -> Result<(), CommonError> {
        let shard_key = key_segment(cluster_name, shard_name, segment_seq);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), shard_key);
    }
}
