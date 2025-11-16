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

use crate::storage::keys::{key_offset, key_offset_by_group};
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_data::{
    engine_delete_by_meta_data, engine_prefix_list_by_meta_data, engine_save_by_meta_data,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct OffsetData {
    pub cluster_name: String,
    pub group: String,
    pub namespace: String,
    pub shard_name: String,
    pub offset: u64,
    pub timestamp: u64,
}

pub struct OffsetStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl OffsetStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        OffsetStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(
        &self,
        cluster_name: &str,
        group: &str,
        namespace: &str,
        shard_name: &str,
        offset: u64,
    ) -> Result<(), CommonError> {
        let key = key_offset(cluster_name, group, namespace, shard_name);
        let offset_data = OffsetData {
            cluster_name: cluster_name.to_owned(),
            group: group.to_owned(),
            namespace: namespace.to_owned(),
            shard_name: shard_name.to_owned(),
            offset,
            timestamp: now_second(),
        };
        engine_save_by_meta_data(self.rocksdb_engine_handler.clone(), &key, offset_data)
    }

    pub fn delete(
        &self,
        cluster_name: &str,
        group: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Result<(), CommonError> {
        let key = key_offset(cluster_name, group, namespace, shard_name);
        engine_delete_by_meta_data(self.rocksdb_engine_handler.clone(), &key)
    }

    pub fn group_offset(
        &self,
        cluster_name: &str,
        group: &str,
    ) -> Result<Vec<OffsetData>, CommonError> {
        let prefix_key = key_offset_by_group(cluster_name, group);

        let data = engine_prefix_list_by_meta_data::<OffsetData>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;

        Ok(data.iter().map(|row| row.data.clone()).collect())
    }
}

#[cfg(test)]
mod test {

    use crate::storage::placement::offset::OffsetStorage;
    use rocksdb_engine::rocksdb::RocksDBEngine;
    use rocksdb_engine::storage::family::column_family_list;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn offset_storage_test() {
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            tempdir().unwrap().path().to_str().unwrap(),
            100,
            column_family_list(),
        ));
        let offset_storage = OffsetStorage::new(rocksdb_engine);

        let cluster_name = "cluster1".to_string();
        let group = "group1".to_string();
        let namespace1 = "namespace1".to_string();
        let namespace2 = "namespace2".to_string();
        let shard_name = "shard1".to_string();

        offset_storage
            .save(&cluster_name, &group, &namespace1, &shard_name, 100)
            .unwrap();
        offset_storage
            .save(&cluster_name, &group, &namespace2, &shard_name, 200)
            .unwrap();

        let offset_list1 = offset_storage.group_offset(&cluster_name, &group).unwrap();
        assert_eq!(offset_list1.len(), 2);
        assert_eq!(offset_list1[1].offset, 200);

        offset_storage
            .delete(&cluster_name, &group, &namespace2, &shard_name)
            .unwrap();

        let offset_list2 = offset_storage.group_offset(&cluster_name, &group).unwrap();
        assert_eq!(offset_list2.len(), 1);
        assert_eq!(offset_list2[0].offset, 100);
    }
}
