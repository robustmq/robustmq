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
use metadata_struct::journal::shard::JournalShard;

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{
    key_all_shard, key_shard, key_shard_cluster_prefix, key_shard_namespace_prefix,
};
use crate::storage::rocksdb::RocksDBEngine;

pub struct ShardStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShardStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ShardStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, shard_info: &JournalShard) -> Result<(), CommonError> {
        let shard_key = key_shard(
            &shard_info.cluster_name,
            &shard_info.namespace,
            &shard_info.shard_name,
        );
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), shard_key, shard_info)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Result<Option<JournalShard>, CommonError> {
        let shard_key: String = key_shard(cluster_name, namespace, shard_name);
        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), shard_key)? {
            return Ok(Some(serde_json::from_str::<JournalShard>(&data.data)?));
        }
        Ok(None)
    }

    pub fn delete(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Result<(), CommonError> {
        let shard_key = key_shard(cluster_name, namespace, shard_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), shard_key)
    }

    pub fn all_shard(&self) -> Result<Vec<JournalShard>, CommonError> {
        let prefix_key = key_all_shard();
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;

        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalShard>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_cluster_namespace(
        &self,
        cluster_name: &str,
        namespace: &str,
    ) -> Result<Vec<JournalShard>, CommonError> {
        let prefix_key = key_shard_namespace_prefix(cluster_name, namespace);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;

        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalShard>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_cluster(&self, cluster_name: &str) -> Result<Vec<JournalShard>, CommonError> {
        let prefix_key = key_shard_cluster_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;

        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalShard>(&raw.data)?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use common_base::tools::unique_id;
    use metadata_struct::journal::shard::{JournalShard, JournalShardStatus};
    use rocksdb_engine::RocksDBEngine;
    use std::sync::Arc;
    use tempfile::tempdir;

    use super::ShardStorage;

    #[test]
    fn shard_storage_test() {
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            tempdir().unwrap().path().to_str().unwrap(),
            100,
            vec!["cluster".to_string()],
        ));

        let num_clusters = 5;
        let num_namespace_per_cluster = 5;
        let num_shards_per_namespace = 10;

        let shard_storage = ShardStorage::new(rocksdb_engine.clone());

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
                    let segment = JournalShard {
                        shard_uid: unique_id(),
                        cluster_name: cluster.clone(),
                        namespace: namespace.clone(),
                        shard_name: shard.clone(),
                        start_segment_seq: 0,
                        active_segment_seq: 0,
                        last_segment_seq: 0,
                        status: JournalShardStatus::Run,
                        ..Default::default()
                    };

                    shard_storage.save(&segment.clone()).unwrap();
                }
            }
        }
        let all_segs = shard_storage.all_shard().unwrap();
        assert_eq!(
            all_segs.len() as u32,
            num_clusters * num_namespace_per_cluster * num_shards_per_namespace
        );

        for cluster in clusters.iter() {
            let cluster_segs = shard_storage.list_by_cluster(cluster).unwrap();
            assert_eq!(
                cluster_segs.len() as u32,
                num_namespace_per_cluster * num_shards_per_namespace
            );

            for namespace in namespaces.iter() {
                let namespace_segs = shard_storage
                    .list_by_cluster_namespace(cluster, namespace)
                    .unwrap();
                assert_eq!(namespace_segs.len() as u32, num_shards_per_namespace);
                for shard in shards.iter() {
                    let shard_segs = shard_storage
                        .get(cluster, namespace, shard)
                        .unwrap()
                        .unwrap();
                    assert_eq!(shard_segs.cluster_name, *cluster);
                    assert_eq!(shard_segs.namespace, *namespace);
                    assert_eq!(shard_segs.shard_name, *shard);
                }
            }
        }
        shard_storage
            .delete(
                clusters[0].as_str(),
                namespaces[0].as_str(),
                shards[0].as_str(),
            )
            .unwrap();
        let all_segs = shard_storage.all_shard().unwrap();
        assert_eq!(
            all_segs.len() as u32,
            num_clusters * num_namespace_per_cluster * num_shards_per_namespace - 1
        );
    }
}
