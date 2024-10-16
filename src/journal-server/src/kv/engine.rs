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

use common_base::config::journal_server::JournalServerConfig;
use common_base::error::journal_server::JournalServerError;
use dashmap::DashMap;
use rocksdb_engine::RocksDBEngine;

use super::rocksdb::{column_family_list, kv_storage_data_fold, DB_COLUMN_FAMILY_DEFAULT};
use crate::core::record::KvRecord;

pub struct KvEngine {
    rocksdb_instances: DashMap<String, RocksDBEngine>,
}

impl KvEngine {
    pub fn new() -> Self {
        let instances = DashMap::with_capacity(2);
        KvEngine {
            rocksdb_instances: instances,
        }
    }

    fn build_instance(&self, config: &JournalServerConfig) {
        for fold in config.storage.data_path.clone() {
            let instance = RocksDBEngine::new(
                &kv_storage_data_fold(&fold),
                config.storage.rocksdb_max_open_files.unwrap(),
                column_family_list(),
            );
            self.rocksdb_instances.insert(fold, instance);
        }
    }

    pub fn set(&self, fold: &String, key: &str, value: KvRecord) -> Result<(), JournalServerError> {
        if let Some(instance) = self.rocksdb_instances.get(fold) {
            let cf = instance.cf_handle(DB_COLUMN_FAMILY_DEFAULT).unwrap();
            return Ok(instance.write(cf, key, &value)?);
        }
        Err(JournalServerError::NoRocksdbInstanceAvailable(
            fold.to_string(),
        ))
    }

    pub fn delete(&self, fold: &String, key: &str) -> Result<(), JournalServerError> {
        if let Some(instance) = self.rocksdb_instances.get(fold) {
            let cf = instance.cf_handle(DB_COLUMN_FAMILY_DEFAULT).unwrap();
            return Ok(instance.delete(cf, key)?);
        }
        Err(JournalServerError::NoRocksdbInstanceAvailable(
            fold.to_string(),
        ))
    }

    pub fn exists(&self, fold: &String, key: &str) -> Result<bool, JournalServerError> {
        if let Some(instance) = self.rocksdb_instances.get(fold) {
            let cf = instance.cf_handle(DB_COLUMN_FAMILY_DEFAULT).unwrap();
            return Ok(instance.exist(cf, key));
        }
        Err(JournalServerError::NoRocksdbInstanceAvailable(
            fold.to_string(),
        ))
    }

    pub fn get(&self, fold: &String, key: &str) -> Result<Option<KvRecord>, JournalServerError> {
        if let Some(instance) = self.rocksdb_instances.get(fold) {
            let cf = instance.cf_handle(DB_COLUMN_FAMILY_DEFAULT).unwrap();
            return Ok(instance.read::<KvRecord>(cf, key)?);
        }
        Err(JournalServerError::NoRocksdbInstanceAvailable(
            fold.to_string(),
        ))
    }
}
