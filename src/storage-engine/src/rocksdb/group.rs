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
    core::{error::StorageEngineError, shard::StorageEngineRunType},
    rocksdb::engine::RocksDBStorageEngine,
};
use common_base::utils::serialize::{deserialize, serialize};
use metadata_struct::storage::adapter_offset::AdapterConsumerGroupOffset;
use rocksdb::WriteBatch;
use rocksdb_engine::keys::storage::{group_record_offsets_key, group_record_offsets_key_prefix};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
struct OffsetInfo {
    pub group_name: String,
    pub shard_name: String,
    pub offset: u64,
}

impl RocksDBStorageEngine {
    pub async fn get_offset_by_group(
        &self,
        group_name: &str,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, StorageEngineError> {
        match self.engine_type {
            StorageEngineRunType::Standalone => {
                let cf = self.get_cf()?;
                let group_record_offsets_key_prefix = group_record_offsets_key_prefix(group_name);

                let mut offsets = Vec::new();
                for (_, v) in self
                    .rocksdb_engine_handler
                    .read_prefix(cf, &group_record_offsets_key_prefix)?
                {
                    let info = deserialize::<OffsetInfo>(&v)?;
                    offsets.push(AdapterConsumerGroupOffset {
                        group: info.group_name,
                        shard_name: info.shard_name,
                        offset: info.offset,
                        ..Default::default()
                    });
                }

                Ok(offsets)
            }
            StorageEngineRunType::EngineStorage => {
                let res = self.offset_manager.get_offset(group_name).await?;
                Ok(res)
            }
        }
    }

    pub async fn commit_offset(
        &self,
        group_name: &str,
        offsets: &HashMap<String, u64>,
    ) -> Result<(), StorageEngineError> {
        if offsets.is_empty() {
            return Ok(());
        }

        match self.engine_type {
            StorageEngineRunType::Standalone => {
                let cf = self.get_cf()?;
                let mut batch = WriteBatch::default();

                for (shard_name, offset) in offsets.iter() {
                    let group_record_offsets_key = group_record_offsets_key(group_name, shard_name);
                    let info = OffsetInfo {
                        group_name: group_name.to_string(),
                        shard_name: shard_name.to_string(),
                        offset: *offset,
                    };
                    batch.put_cf(&cf, group_record_offsets_key.as_bytes(), serialize(&info)?);
                }

                self.rocksdb_engine_handler.write_batch(batch)?;
            }
            StorageEngineRunType::EngineStorage => {
                self.offset_manager
                    .commit_offset(group_name, offsets)
                    .await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::shard::StorageEngineRunType;
    use crate::core::test_tool::test_build_engine;
    use common_base::tools::unique_id;

    #[tokio::test]
    async fn test_standalone_commit_and_get() {
        let engine = test_build_engine(StorageEngineRunType::Standalone);
        let group_name = unique_id();
        let shard1 = unique_id();
        let shard2 = unique_id();

        let mut offsets = HashMap::new();
        offsets.insert(shard1.clone(), 100);
        offsets.insert(shard2.clone(), 200);

        engine.commit_offset(&group_name, &offsets).await.unwrap();

        let result = engine.get_offset_by_group(&group_name).await.unwrap();
        assert_eq!(result.len(), 2);
        assert!(result
            .iter()
            .any(|o| o.shard_name == shard1 && o.offset == 100));
        assert!(result
            .iter()
            .any(|o| o.shard_name == shard2 && o.offset == 200));
    }
}
