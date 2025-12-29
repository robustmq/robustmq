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

use crate::offset::storage::OffsetStorageManager;
use common_base::{
    error::common::CommonError,
    utils::serialize::{deserialize, serialize},
};
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::adapter_offset::{AdapterOffsetStrategy, AdapterReadShardOffset};
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
        let offset_storage = OffsetStorageManager::new(client_pool);
        OffsetCacheManager {
            rocksdb_engine_handler,
            group_update_flag: DashMap::with_capacity(64),
            offset_storage,
        }
    }

    pub async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let mut batch = rocksdb::WriteBatch::default();
        let cf = get_cf_handle(&self.rocksdb_engine_handler, DB_COLUMN_FAMILY_BROKER)?;
        for (shard_name, offset) in offset.iter() {
            let key = self.offset_key(group_name, shard_name);
            let shard_offset = AdapterReadShardOffset {
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
        let groups: DashMap<String, Vec<AdapterReadShardOffset>> = DashMap::new();

        // Only read offsets for groups with pending updates (flag=true)
        {
            let cf = get_cf_handle(&self.rocksdb_engine_handler, DB_COLUMN_FAMILY_BROKER)?;

            for entry in self.group_update_flag.iter() {
                let group_name = entry.key();
                let flag = *entry.value();

                if !flag {
                    continue;
                }

                let key_prefix = self.offset_group_key_prefix(group_name);
                let mut group_offsets = Vec::new();

                for (_, val) in self
                    .rocksdb_engine_handler
                    .read_prefix(cf.clone(), &key_prefix)?
                {
                    let data = deserialize::<AdapterReadShardOffset>(&val)?;
                    group_offsets.push(data);
                }

                if !group_offsets.is_empty() {
                    groups.insert(group_name.clone(), group_offsets);
                }
            }
        }

        let updates: DashMap<String, Vec<AdapterReadShardOffset>> = DashMap::new();

        for group_raw in groups.iter() {
            let group = group_raw.key();
            let local_offsets = group_raw.value();
            let remote_offsets = self
                .offset_storage
                .get_offset(group, AdapterOffsetStrategy::Earliest)
                .await?;

            let mut remote_map = HashMap::new();
            for remote_offset in remote_offsets {
                remote_map.insert(remote_offset.shard_name.clone(), remote_offset);
            }

            let mut group_updates = Vec::new();
            for local_offset in local_offsets.iter() {
                if let Some(remote_offset) = remote_map.get(&local_offset.shard_name) {
                    if local_offset.offset > remote_offset.offset {
                        group_updates.push(local_offset.clone());
                    }
                } else {
                    // New shard not in remote storage, add it
                    group_updates.push(local_offset.clone());
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

        // Reset flags after successful commit, but preserve flags set during commit
        // Use compare-and-swap to avoid race conditions
        for entry in offsets.iter() {
            let group_name = entry.key().clone();
            self.group_update_flag.entry(group_name).and_modify(|flag| {
                // Only reset if still true (not modified by concurrent commit)
                if *flag {
                    *flag = false;
                }
            });
        }

        Ok(())
    }

    pub async fn get_local_offset(
        &self,
    ) -> Result<DashMap<String, Vec<AdapterReadShardOffset>>, CommonError> {
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
                let data = deserialize::<AdapterReadShardOffset>(&val)?;
                group_data.push(data);
            }

            if !group_data.is_empty() {
                results.insert(group_name.clone(), group_data);
            }
        }
        Ok(results)
    }

    fn offset_key(&self, group_name: &str, shard_name: &str) -> String {
        format!("/offset/{group_name}/{shard_name}")
    }

    fn offset_group_key_prefix(&self, group_name: &str) -> String {
        format!("/offset/{group_name}/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grpc_clients::pool::ClientPool;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn setup() -> OffsetCacheManager {
        OffsetCacheManager::new(test_rocksdb_instance(), Arc::new(ClientPool::new(10)))
    }

    #[tokio::test]
    async fn test_basic_commit_and_retrieval() {
        let cache = setup();

        // Commit multiple shards for one group
        cache
            .commit_offset(
                "g1",
                &HashMap::from([("s1".into(), 100u64), ("s2".into(), 200u64)]),
            )
            .await
            .unwrap();

        // Verify flag set and offsets retrievable
        assert!(cache.group_update_flag.get("g1").is_some_and(|v| *v));
        let local = cache.get_local_offset().await.unwrap();
        assert_eq!(local.get("g1").expect("g1 exists").len(), 2);
    }

    #[tokio::test]
    async fn test_flag_based_filtering() {
        let cache = setup();

        cache
            .commit_offset("g1", &HashMap::from([("s1".into(), 10u64)]))
            .await
            .unwrap();
        cache
            .commit_offset("g2", &HashMap::from([("s2".into(), 20u64)]))
            .await
            .unwrap();

        cache.group_update_flag.insert("g2".into(), false);

        let local = cache.get_local_offset().await.unwrap();
        assert_eq!(local.len(), 1);
        assert!(local.contains_key("g1"));
    }
}
