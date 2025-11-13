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

use crate::storage::keys::{key_cluster, key_cluster_prefix};
use common_base::error::common::CommonError;
use metadata_struct::meta::cluster::ClusterInfo;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};
use std::sync::Arc;

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
        let key = key_cluster(&cluster_info.cluster_name);
        engine_save_by_meta_metadata(
            self.rocksdb_engine_handler.clone(),
            &key,
            cluster_info.clone(),
        )
    }

    pub fn list(&self) -> Result<Vec<ClusterInfo>, CommonError> {
        let prefix_key = key_cluster_prefix();
        let data = engine_prefix_list_by_meta_metadata::<ClusterInfo>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        let mut results = Vec::new();
        for raw in data {
            results.push(raw.data);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod test {
    use metadata_struct::meta::cluster::ClusterInfo;
    use rocksdb_engine::rocksdb::RocksDBEngine;
    use rocksdb_engine::storage::family::column_family_list;
    use std::sync::Arc;
    use tempfile::tempdir;

    use super::ClusterStorage;

    #[test]
    fn cluster_storage_test() {
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            tempdir().unwrap().path().to_str().unwrap(),
            100,
            column_family_list(),
        ));
        let cluster_storage = ClusterStorage::new(rocksdb_engine);

        for i in 0..10 {
            let cluster_info = ClusterInfo {
                cluster_name: format!("cluster_{i}"),
                ..Default::default()
            };

            cluster_storage.save(&cluster_info).unwrap();
        }

        let all_clusters = cluster_storage.list().unwrap();
        assert_eq!(all_clusters.len(), 10);
    }
}
