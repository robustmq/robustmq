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

use common_base::error::common::CommonError;
use dashmap::DashMap;
use rocksdb_engine::{
    rocksdb::RocksDBEngine,
    storage::{base::get_cf_handle, family::DB_COLUMN_FAMILY_BROKER},
};
use std::{collections::HashMap, sync::Arc};

pub struct OffsetCache {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    group_update_flag: DashMap<String, bool>,
}

impl OffsetCache {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        OffsetCache {
            rocksdb_engine_handler,
            group_update_flag: DashMap::with_capacity(4),
        }
    }

    pub async fn commit_offset(
        &self,
        group_name: &str,
        namespace: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let mut batch = rocksdb::WriteBatch::default();
        let cf = get_cf_handle(&self.rocksdb_engine_handler, DB_COLUMN_FAMILY_BROKER)?;
        for (shard_name, offset) in offset.iter() {
            let key = self.offset_key(group_name, namespace, shard_name);
            batch.put_cf(&cf, &key, offset.to_le_bytes());
            self.group_update_flag.insert(key, true);
        }
        self.rocksdb_engine_handler.db.write(batch)?;
        Ok(())
    }

    pub async fn flush(&self) -> Result<(), CommonError> {
        Ok(())
    }

    pub fn async_commit_offset_to_storage(&self) -> Result<(), CommonError> {
        Ok(())
    }

    async fn get_local_offset(&self) {}

    fn offset_key(&self, group_name: &str, namespace: &str, shard_name: &str) -> String {
        format!("/offset/{}/{}/{}", group_name, namespace, shard_name)
    }
}
