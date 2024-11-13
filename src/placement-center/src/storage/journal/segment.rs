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

use std::sync::Arc;

use common_base::error::common::CommonError;
use metadata_struct::journal::segment::{JournalSegment, SegmentStatus};

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{
    key_all_segment, key_segment, key_segment_cluster_prefix, key_segment_namespace_prefix,
    key_segment_shard_prefix,
};
use crate::storage::rocksdb::RocksDBEngine;

#[allow(dead_code)]
pub fn is_seal_up_segment(status: &SegmentStatus) -> bool {
    *status == SegmentStatus::PreSealUp || *status == SegmentStatus::SealUp
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

    pub fn save(&self, segment: JournalSegment) -> Result<(), CommonError> {
        let shard_key = key_segment(
            &segment.cluster_name,
            &segment.namespace,
            &segment.shard_name,
            segment.segment_seq,
        );
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), shard_key, segment)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        segment_seq: u32,
    ) -> Result<Option<JournalSegment>, CommonError> {
        let shard_key: String = key_segment(cluster_name, namespace, shard_name, segment_seq);

        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), shard_key) {
            Ok(Some(data)) => match serde_json::from_slice::<JournalSegment>(&data.data) {
                Ok(segment) => Ok(Some(segment)),
                Err(e) => Err(e.into()),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn all_segment(&self) -> Result<Vec<JournalSegment>, CommonError> {
        let prefix_key = key_all_segment();
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<JournalSegment>(&raw.data) {
                        Ok(topic) => {
                            results.push(topic);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }

    pub fn list_by_cluster(&self, cluster_name: &str) -> Result<Vec<JournalSegment>, CommonError> {
        let prefix_key = key_segment_cluster_prefix(cluster_name);
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<JournalSegment>(&raw.data) {
                        Ok(topic) => {
                            results.push(topic);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }

    pub fn list_by_namespace(
        &self,
        cluster_name: &str,
        namespace: &str,
    ) -> Result<Vec<JournalSegment>, CommonError> {
        let prefix_key = key_segment_namespace_prefix(cluster_name, namespace);
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<JournalSegment>(&raw.data) {
                        Ok(topic) => {
                            results.push(topic);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }

    pub fn list_by_shard(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Result<Vec<JournalSegment>, CommonError> {
        let prefix_key = key_segment_shard_prefix(cluster_name, namespace, shard_name);
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<JournalSegment>(&raw.data) {
                        Ok(topic) => {
                            results.push(topic);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }

    pub fn delete(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        segment_seq: u32,
    ) -> Result<(), CommonError> {
        let shard_key = key_segment(cluster_name, namespace, shard_name, segment_seq);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), shard_key)
    }
}
