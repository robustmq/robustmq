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

use crate::{
    core::{error::StorageEngineError, shard::ShardState},
    rocksdb::engine::RocksDBStorageEngine,
};
use common_base::utils::serialize::{self};
use metadata_struct::storage::adapter_offset::AdapterShardInfo;
use rocksdb_engine::keys::storage::{
    key_index_prefix, shard_info_key, shard_info_key_prefix, shard_record_key_prefix,
    tag_index_prefix, timestamp_index_prefix,
};
use std::sync::Arc;

impl RocksDBStorageEngine {
    pub async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), StorageEngineError> {
        self.storage_type_check()?;

        let shard_name = &shard.shard_name;
        let cf = self.get_cf()?;
        let shard_info_key = shard_info_key(shard_name);

        if self
            .rocksdb_engine_handler
            .exist(cf.clone(), &shard_info_key)
        {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "shard {shard_name} already exists"
            )));
        }

        self.rocksdb_engine_handler
            .write(cf.clone(), &shard_info_key, &shard)?;

        self.shard_state
            .insert(shard.shard_name.to_string(), ShardState::default());
        self.shard_write_locks
            .entry(shard_name.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())));

        Ok(())
    }

    pub async fn list_shard(
        &self,
        shard: Option<String>,
    ) -> Result<Vec<AdapterShardInfo>, StorageEngineError> {
        self.storage_type_check()?;

        let cf = self.get_cf()?;
        if let Some(shard_name) = shard {
            let key = shard_info_key(&shard_name);
            if let Some(v) = self
                .rocksdb_engine_handler
                .read::<AdapterShardInfo>(cf.clone(), &key)?
            {
                Ok(vec![v])
            } else {
                Ok(Vec::new())
            }
        } else {
            let raw_shard_info = self
                .rocksdb_engine_handler
                .read_prefix(cf.clone(), &shard_info_key_prefix())?;
            let mut result = Vec::new();
            for (_, v) in raw_shard_info {
                result.push(serialize::deserialize::<AdapterShardInfo>(v.as_slice())?);
            }
            Ok(result)
        }
    }

    pub async fn delete_shard(&self, shard: &str) -> Result<(), StorageEngineError> {
        self.storage_type_check()?;

        let cf = self.get_cf()?;
        let shard_info_key = shard_info_key(shard);

        if !self
            .rocksdb_engine_handler
            .exist(cf.clone(), &shard_info_key)
        {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "shard {shard} does not exist"
            )));
        }

        // delete records
        let record_prefix = shard_record_key_prefix(shard);
        self.rocksdb_engine_handler
            .delete_prefix(cf.clone(), &record_prefix)?;

        // delete key index
        let key_prefix = key_index_prefix(shard);
        self.rocksdb_engine_handler
            .delete_prefix(cf.clone(), &key_prefix)?;

        // delete tag index
        let tag_index_prefix = tag_index_prefix(shard);
        self.rocksdb_engine_handler
            .delete_prefix(cf.clone(), &tag_index_prefix)?;

        // delete timestamp index
        let timestamp_index_prefix = timestamp_index_prefix(shard);
        self.rocksdb_engine_handler
            .delete_prefix(cf.clone(), &timestamp_index_prefix)?;

        // delete shard info
        self.rocksdb_engine_handler.delete(cf, &shard_info_key)?;

        // remove lock
        self.shard_write_locks.remove(shard);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::shard::StorageEngineRunType;
    use crate::core::test_tool::test_build_engine;
    use common_base::tools::unique_id;
    use metadata_struct::storage::adapter_offset::AdapterShardInfo;

    #[tokio::test]
    async fn test_engine_storage_type_error() {
        let engine = test_build_engine(StorageEngineRunType::EngineStorage);
        let shard = AdapterShardInfo {
            shard_name: unique_id(),
            ..Default::default()
        };
        assert!(engine.create_shard(&shard).await.is_err());
        assert!(engine.list_shard(None).await.is_err());
        assert!(engine.delete_shard(&shard.shard_name).await.is_err());
    }

    #[tokio::test]
    async fn test_standalone_shard_operations() {
        let engine = test_build_engine(StorageEngineRunType::Standalone);
        let shard_name = unique_id();
        let shard = AdapterShardInfo {
            shard_name: shard_name.clone(),
            ..Default::default()
        };

        engine.create_shard(&shard).await.unwrap();
        let list1 = engine.list_shard(None).await.unwrap();
        assert!(!list1.is_empty());

        engine.delete_shard(&shard_name).await.unwrap();
        let list2 = engine.list_shard(Some(shard_name)).await.unwrap();
        assert!(list2.is_empty());
    }
}
