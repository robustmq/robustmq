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

use crate::{offset::storage::OffsetStorageManager, storage::ShardOffset};
use common_base::{
    error::common::CommonError,
    utils::serialize::{deserialize, serialize},
};
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use rocksdb_engine::{
    rocksdb::RocksDBEngine,
    storage::{base::get_cf_handle, family::DB_COLUMN_FAMILY_BROKER},
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct OffsetCacheManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    group_update_flag: DashMap<String, bool>,
    offset_storage: OffsetStorageManager,
}

impl OffsetCacheManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>, client_pool: Arc<ClientPool>) -> Self {
        let offset_storage = OffsetStorageManager::new(client_pool.clone());
        OffsetCacheManager {
            rocksdb_engine_handler,
            group_update_flag: DashMap::with_capacity(64),
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
                shard_name: shard_name.to_string(),
                offset: *offset,
                ..Default::default()
            };

            let val = serialize(&shard_offset)?;
            batch.put_cf(&cf, &key, val);
        }
        self.rocksdb_engine_handler.db.write(batch)?;
        self.group_update_flag.insert(group_name.to_string(), true);
        Ok(())
    }

    pub async fn flush(&self) -> Result<(), CommonError> {
        self.async_commit_offset_to_storage().await
    }

    pub async fn try_comparison_and_save_offset(&self) -> Result<(), CommonError> {
        let key_prefix = self.offset_key_prefix();

        let groups: DashMap<String, Vec<ShardOffset>> = DashMap::new();

        {
            let cf = get_cf_handle(&self.rocksdb_engine_handler, DB_COLUMN_FAMILY_BROKER)?;
            for (_, val) in self
                .rocksdb_engine_handler
                .read_prefix(cf.clone(), &key_prefix)?
            {
                let data = deserialize::<ShardOffset>(&val)?;
                if let Some(mut raw) = groups.get_mut(&data.group) {
                    raw.push(data);
                } else {
                    groups.insert(data.group.clone(), vec![data]);
                }
            }
        }

        let updates: DashMap<String, Vec<ShardOffset>> = DashMap::new();

        for group_raw in groups.iter() {
            let group = group_raw.key();
            let local_offsets = group_raw.value();
            let remote_offsets = self.offset_storage.get_offset(group).await?;

            let mut remote_map = HashMap::new();
            for remote_offset in remote_offsets {
                let key = format!("{}:{}", remote_offset.namespace, remote_offset.shard_name);
                remote_map.insert(key, remote_offset);
            }

            let mut group_updates = Vec::new();
            for local_offset in local_offsets.iter() {
                let key = format!("{}:{}", local_offset.namespace, local_offset.shard_name);

                if let Some(remote_offset) = remote_map.get(&key) {
                    if local_offset.offset > remote_offset.offset {
                        group_updates.push(local_offset.clone());
                    }
                }
            }

            if !group_updates.is_empty() {
                updates.insert(group.clone(), group_updates);
            }
        }

        if !updates.is_empty() {
            self.offset_storage.batch_commit_offset(&updates).await?;
        }

        Ok(())
    }

    pub async fn async_commit_offset_to_storage(&self) -> Result<(), CommonError> {
        let offsets = self.get_local_offset().await?;
        if offsets.is_empty() {
            return Ok(());
        }

        self.offset_storage.batch_commit_offset(&offsets).await?;

        // Reset flags after successful commit
        for entry in offsets.iter() {
            self.group_update_flag.insert(entry.key().clone(), false);
        }

        Ok(())
    }

    pub async fn get_local_offset(&self) -> Result<DashMap<String, Vec<ShardOffset>>, CommonError> {
        let results = DashMap::with_capacity(16);
        let cf = get_cf_handle(&self.rocksdb_engine_handler, DB_COLUMN_FAMILY_BROKER)?;

        for entry in self.group_update_flag.iter() {
            let group_name = entry.key();
            let flag = *entry.value();

            if !flag {
                continue;
            }

            let key_prefix = self.offset_group_key_prefix(group_name);
            let mut group_data = Vec::new();
            for (_, val) in self
                .rocksdb_engine_handler
                .read_prefix(cf.clone(), &key_prefix)?
            {
                let data = deserialize::<ShardOffset>(&val)?;
                group_data.push(data);
            }

            if !group_data.is_empty() {
                results.insert(group_name.clone(), group_data);
            }
        }
        Ok(results)
    }

    fn offset_key(&self, group_name: &str, namespace: &str, shard_name: &str) -> String {
        format!("/offset/{group_name}/{namespace}/{shard_name}")
    }

    fn offset_group_key_prefix(&self, group_name: &str) -> String {
        format!("/offset/{group_name}/")
    }

    fn offset_key_prefix(&self) -> String {
        "/offset/".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grpc_clients::pool::ClientPool;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_commit_and_get_local_offset() {
        let rocksdb = test_rocksdb_instance();
        let client_pool = Arc::new(ClientPool::new(10));
        let cache = OffsetCacheManager::new(rocksdb, client_pool);

        let group_name = "test_group";
        let namespace = "test_namespace";
        let mut offsets = HashMap::new();
        offsets.insert("shard_1".to_string(), 100u64);
        offsets.insert("shard_2".to_string(), 200u64);

        // Test commit_offset
        cache
            .commit_offset(group_name, namespace, &offsets)
            .await
            .unwrap();

        // Verify flag is set
        assert_eq!(
            cache.group_update_flag.get(group_name).map(|r| *r.value()),
            Some(true)
        );

        // Test get_local_offset
        let local_offsets = cache.get_local_offset().await.unwrap();
        assert_eq!(local_offsets.len(), 1);
        assert!(local_offsets.contains_key(group_name));

        let group_offsets = local_offsets.get(group_name).unwrap();
        assert_eq!(group_offsets.len(), 2);

        // Verify offset values
        for shard_offset in group_offsets.iter() {
            assert_eq!(shard_offset.namespace, namespace);
            match shard_offset.shard_name.as_str() {
                "shard_1" => assert_eq!(shard_offset.offset, 100),
                "shard_2" => assert_eq!(shard_offset.offset, 200),
                _ => panic!("Unexpected shard name"),
            }
        }
    }
}
