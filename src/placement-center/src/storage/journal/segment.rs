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
        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), shard_key)? {
            return Ok(Some(serde_json::from_str::<JournalSegment>(&data.data)?));
        }
        Ok(None)
    }

    pub fn all_segment(&self) -> Result<Vec<JournalSegment>, CommonError> {
        let prefix_key = key_all_segment();
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalSegment>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_cluster(&self, cluster_name: &str) -> Result<Vec<JournalSegment>, CommonError> {
        let prefix_key = key_segment_cluster_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalSegment>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_namespace(
        &self,
        cluster_name: &str,
        namespace: &str,
    ) -> Result<Vec<JournalSegment>, CommonError> {
        let prefix_key = key_segment_namespace_prefix(cluster_name, namespace);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalSegment>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_shard(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Result<Vec<JournalSegment>, CommonError> {
        let prefix_key = key_segment_shard_prefix(cluster_name, namespace, shard_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalSegment>(&raw.data)?);
        }
        Ok(results)
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

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use common_base::tools::unique_id;
    use metadata_struct::journal::segment::JournalSegment;
    use rocksdb_engine::RocksDBEngine;
    use tempfile::tempdir;

    use super::SegmentStorage;

    #[test]
    fn segment_store_test() {
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            tempdir().unwrap().path().to_str().unwrap(),
            100,
            vec!["cluster".to_string()],
        ));

        let segs_per_shard = 5;
        let num_clusters = 5;
        let num_namespace_per_cluster = 5;
        let num_shards_per_namespace = 10;

        let segment_storage = SegmentStorage::new(rocksdb_engine.clone());

        let clusters = (0..num_clusters)
            .map(|i| format!("cluster_{}", i))
            .collect::<Vec<_>>();

        let namespaces = (0..num_namespace_per_cluster)
            .map(|_| unique_id())
            .collect::<Vec<_>>();

        let shards = (0..num_shards_per_namespace)
            .map(|i| format!("shard_{}", i))
            .collect::<Vec<_>>();

        for cluster in clusters.iter() {
            for namespace in namespaces.iter() {
                for shard in shards.iter() {
                    // 5 segments per shard
                    for seq in 0..segs_per_shard {
                        let segment = JournalSegment {
                            cluster_name: cluster.clone(),
                            namespace: namespace.clone(),
                            shard_name: shard.clone(),
                            segment_seq: seq,
                            ..Default::default()
                        };

                        segment_storage.save(segment.clone()).unwrap();
                    }
                }
            }
        }

        let all_segs = segment_storage.all_segment().unwrap();

        assert_eq!(
            all_segs.len() as u32,
            num_clusters * num_namespace_per_cluster * num_shards_per_namespace * segs_per_shard
        );

        for cluster in clusters.iter() {
            let cluster_segs = segment_storage.list_by_cluster(cluster).unwrap();
            assert_eq!(
                cluster_segs.len() as u32,
                num_namespace_per_cluster * num_shards_per_namespace * segs_per_shard
            );

            for namespace in namespaces.iter() {
                let namespace_segs = segment_storage
                    .list_by_namespace(cluster, namespace)
                    .unwrap();
                assert_eq!(
                    namespace_segs.len() as u32,
                    num_shards_per_namespace * segs_per_shard
                );

                for shard in shards.iter() {
                    let shard_segs = segment_storage
                        .list_by_shard(cluster, namespace, shard)
                        .unwrap();
                    assert_eq!(shard_segs.len() as u32, segs_per_shard);

                    for seq in 0..segs_per_shard {
                        let seg = segment_storage
                            .get(cluster, namespace, shard, seq)
                            .unwrap()
                            .unwrap();
                        assert_eq!(seg.cluster_name, *cluster);
                        assert_eq!(seg.namespace, *namespace);
                        assert_eq!(seg.shard_name, *shard);
                        assert_eq!(seg.segment_seq, seq);
                    }
                }
            }
        }

        for cluster in clusters.iter() {
            for namespace in namespaces.iter() {
                for shard in shards.iter() {
                    for seq in 0..segs_per_shard {
                        segment_storage
                            .delete(cluster, namespace, shard, seq)
                            .unwrap();
                    }
                }
            }
        }

        let all_segs = segment_storage.all_segment().unwrap();
        assert_eq!(all_segs.len(), 0);

        for cluster in clusters.iter() {
            let cluster_segs = segment_storage.list_by_cluster(cluster).unwrap();
            assert_eq!(cluster_segs.len(), 0);

            for namespace in namespaces.iter() {
                let namespace_segs = segment_storage
                    .list_by_namespace(cluster, namespace)
                    .unwrap();
                assert_eq!(namespace_segs.len(), 0);

                for shard in shards.iter() {
                    let shard_segs = segment_storage
                        .list_by_shard(cluster, namespace, shard)
                        .unwrap();
                    assert_eq!(shard_segs.len(), 0);

                    for seq in 0..segs_per_shard {
                        assert!(segment_storage
                            .get(cluster, namespace, shard, seq)
                            .unwrap()
                            .is_none());
                    }
                }
            }
        }
    }
}
