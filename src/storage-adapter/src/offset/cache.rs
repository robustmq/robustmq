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

use common_base::{
    error::{common::CommonError, ResultCommonError},
    tools::loop_select_ticket,
    utils::serialize::{deserialize, serialize},
};
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use rocksdb_engine::{
    rocksdb::RocksDBEngine,
    storage::{base::get_cf_handle, family::DB_COLUMN_FAMILY_BROKER},
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;

use crate::{offset::storage::OffsetStorageManager, storage::ShardOffset};

pub struct OffsetCache {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    group_update_flag: DashMap<String, bool>,
    offset_storage: OffsetStorageManager,
}

impl OffsetCache {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>, client_pool: Arc<ClientPool>) -> Self {
        let offset_storage = OffsetStorageManager::new(client_pool.clone());
        OffsetCache {
            rocksdb_engine_handler,
            group_update_flag: DashMap::with_capacity(4),
            offset_storage,
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
            let shard_offset = ShardOffset {
                namespace: namespace.to_string(),
                shard_name: namespace.to_string(),
                offset: *offset,
                ..Default::default()
            };

            let val = serialize(&shard_offset)?;
            batch.put_cf(&cf, &key, val);
            self.group_update_flag.insert(group_name.to_string(), true);
        }
        self.rocksdb_engine_handler.db.write(batch)?;
        Ok(())
    }

    pub async fn flush(&self) -> Result<(), CommonError> {
        self.async_commit_offset_to_storage().await
    }

    pub async fn async_commit_offset_to_storage(&self) -> Result<(), CommonError> {
        let offsets = self.get_local_offset().await?;
        self.offset_storage.batch_commit_offset(&offsets).await
    }

    pub async fn get_local_offset(&self) -> Result<DashMap<String, Vec<ShardOffset>>, CommonError> {
        let results = DashMap::with_capacity(2);
        for (group_name, flag) in self.group_update_flag.clone() {
            if !flag {
                continue;
            }
            let key_prefix = self.offset_key_prefix(&group_name);
            let cf = get_cf_handle(&self.rocksdb_engine_handler, DB_COLUMN_FAMILY_BROKER)?;
            let mut group_data = Vec::new();
            for (_, val) in self.rocksdb_engine_handler.read_prefix(cf, &key_prefix)? {
                let data = deserialize::<ShardOffset>(&val)?;
                group_data.push(data);
            }
            results.insert(group_name, group_data);
        }
        Ok(results)
    }

    fn offset_key(&self, group_name: &str, namespace: &str, shard_name: &str) -> String {
        format!("/offset/{}/{}/{}", group_name, namespace, shard_name)
    }

    fn offset_key_prefix(&self, group_name: &str) -> String {
        format!("/offset/{}/", group_name)
    }
}

pub fn flush_commit_offset_thread(
    offset_cache: Arc<OffsetCache>,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let ac_fn =
            async || -> ResultCommonError { offset_cache.async_commit_offset_to_storage().await };

        loop_select_ticket(ac_fn, 100, &stop_send).await;
    });
}
