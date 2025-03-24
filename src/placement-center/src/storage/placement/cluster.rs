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
use metadata_struct::placement::cluster::ClusterInfo;

use crate::storage::engine::{engine_prefix_list_by_cluster, engine_save_by_cluster};
use crate::storage::keys::{key_cluster, key_cluster_prefix, key_cluster_prefix_by_type};
use crate::storage::rocksdb::RocksDBEngine;

pub struct ClusterStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ClusterStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ClusterStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, cluster_info: &ClusterInfo) -> Result<(), CommonError> {
        let key = key_cluster(&cluster_info.cluster_type, &cluster_info.cluster_name);
        engine_save_by_cluster(
            self.rocksdb_engine_handler.clone(),
            key,
            cluster_info.clone(),
        )
    }

    pub fn list(&self, cluster_type: Option<String>) -> Result<Vec<ClusterInfo>, CommonError> {
        let prefix_key = if let Some(ct) = cluster_type {
            key_cluster_prefix_by_type(&ct)
        } else {
            key_cluster_prefix()
        };
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<ClusterInfo>(&raw.data)?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use metadata_struct::placement::cluster::ClusterInfo;
    use protocol::placement_center::placement_center_inner::ClusterType;
    use rocksdb_engine::RocksDBEngine;
    use tempfile::tempdir;

    use super::ClusterStorage;

    #[test]
    fn cluster_storage_test() {
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            tempdir().unwrap().path().to_str().unwrap(),
            100,
            vec!["cluster".to_string()],
        ));
        let cluster_storage = ClusterStorage::new(rocksdb_engine);

        for i in 0..10 {
            let cluster_type = if i % 2 == 0 {
                ClusterType::PlacementCenter.as_str_name().to_string()
            } else {
                ClusterType::JournalServer.as_str_name().to_string()
            };

            let cluster_info = ClusterInfo {
                cluster_name: format!("cluster_{}", i),
                cluster_type,
                ..Default::default()
            };

            cluster_storage.save(&cluster_info).unwrap();
        }

        let placement_clusters = cluster_storage
            .list(Some(ClusterType::PlacementCenter.as_str_name().to_string()))
            .unwrap();

        assert_eq!(placement_clusters.len(), 5);

        let journal_clusters = cluster_storage
            .list(Some(ClusterType::JournalServer.as_str_name().to_string()))
            .unwrap();
        assert_eq!(journal_clusters.len(), 5);

        let all_clusters = cluster_storage.list(None).unwrap();
        assert_eq!(all_clusters.len(), 10);
    }
}
