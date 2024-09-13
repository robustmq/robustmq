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

use crate::storage::{
    engine::{engine_delete_by_cluster, engine_exists_by_cluster, engine_save_by_cluster},
    keys::key_resource_idempotent,
    rocksdb::RocksDBEngine,
};
use common_base::error::common::CommonError;
use std::sync::Arc;

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
        cluster_name: &String,
        producer_id: &String,
        seq_num: u64,
    ) -> Result<(), CommonError> {
        let key = key_resource_idempotent(cluster_name, producer_id, seq_num);
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, seq_num);
    }

    pub fn delete(
        &self,
        cluster_name: &String,
        producer_id: &String,
        seq_num: u64,
    ) -> Result<(), CommonError> {
        let key = key_resource_idempotent(cluster_name, producer_id, seq_num);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }

    pub fn exists(
        &self,
        cluster_name: &String,
        producer_id: &String,
        seq_num: u64,
    ) -> Result<bool, CommonError> {
        let key = key_resource_idempotent(cluster_name, producer_id, seq_num);
        return engine_exists_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }
}
