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

use crate::storage::engine::{
    engine_delete_by_cluster, engine_exists_by_cluster, engine_save_by_cluster,
};
use crate::storage::keys::key_resource_idempotent;
use crate::storage::rocksdb::RocksDBEngine;

pub struct IdempotentStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl IdempotentStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        IdempotentStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(
        &self,
        cluster_name: &str,
        producer_id: &str,
        seq_num: u64,
    ) -> Result<(), CommonError> {
        let key = key_resource_idempotent(cluster_name, producer_id, seq_num);
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, seq_num)
    }

    pub fn delete(
        &self,
        cluster_name: &str,
        producer_id: &str,
        seq_num: u64,
    ) -> Result<(), CommonError> {
        let key = key_resource_idempotent(cluster_name, producer_id, seq_num);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }

    pub fn exists(
        &self,
        cluster_name: &str,
        producer_id: &str,
        seq_num: u64,
    ) -> Result<bool, CommonError> {
        let key = key_resource_idempotent(cluster_name, producer_id, seq_num);
        engine_exists_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }
}

#[cfg(test)]
mod test {

    use crate::storage::placement::idempotent::IdempotentStorage;
    use crate::storage::rocksdb::RocksDBEngine;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn idempotent_storage_test() {
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            tempdir().unwrap().path().to_str().unwrap(),
            100,
            vec!["cluster".to_string()],
        ));
        let idempotent_storage = IdempotentStorage::new(rocksdb_engine);

        let cluster_name = "cluster1".to_string();
        let producer_id = "producer1".to_string();

        idempotent_storage
            .save(&cluster_name, &producer_id, 100)
            .unwrap();
        idempotent_storage
            .save(&cluster_name, &producer_id, 200)
            .unwrap();

        let exists = idempotent_storage
            .exists(&cluster_name, &producer_id, 100)
            .unwrap();
        assert!(exists);
        let exists = idempotent_storage
            .exists(&cluster_name, &producer_id, 200)
            .unwrap();
        assert!(exists);
        let exists = idempotent_storage
            .exists(&cluster_name, &producer_id, 300)
            .unwrap();
        assert!(!exists);

        idempotent_storage
            .delete(&cluster_name, &producer_id, 100)
            .unwrap();
        let exists = idempotent_storage
            .exists(&cluster_name, &producer_id, 100)
            .unwrap();
        assert!(!exists);
    }
}
