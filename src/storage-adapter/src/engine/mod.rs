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

use crate::storage::StorageAdapter;
use axum::async_trait;
use common_base::error::common::CommonError;
use metadata_struct::storage::adapter_offset::{
    AdapterConsumerGroupOffset, AdapterOffsetStrategy, AdapterReadShardInfo, AdapterShardInfo,
};
use metadata_struct::storage::adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow};
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::storage_record::StorageRecord;
use std::collections::HashMap;
use std::sync::Arc;
use storage_engine::handler::adapter::AdapterHandler;
pub struct StorageEngineAdapter {
    adapter: Arc<AdapterHandler>,
}

impl StorageEngineAdapter {
    pub async fn new(adapter: Arc<AdapterHandler>) -> StorageEngineAdapter {
        StorageEngineAdapter { adapter }
    }
}

#[async_trait]
impl StorageAdapter for StorageEngineAdapter {
    async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), CommonError> {
        self.adapter.create_shard(shard).await
    }

    async fn list_shard(
        &self,
        shard: Option<String>,
    ) -> Result<Vec<AdapterReadShardInfo>, CommonError> {
        self.adapter.list_shard(shard).await
    }

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
        self.adapter.delete_shard(shard).await
    }

    async fn write(
        &self,
        shard: &str,
        record: &AdapterWriteRecord,
    ) -> Result<AdapterWriteRespRow, CommonError> {
        let res = self
            .adapter
            .batch_write(shard, std::slice::from_ref(record))
            .await?;

        if let Some(offset) = res.first() {
            return Ok(offset.clone());
        }

        return Err(CommonError::CommonError(
            "Data writing failed, although it indicated success,
             but the Offset does not exist. Check if there is any error in the logic."
                .to_string(),
        ));
    }

    async fn batch_write(
        &self,
        shard: &str,
        records: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError> {
        self.adapter.batch_write(shard, records).await
    }

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.adapter
            .read_by_offset(shard, offset, read_config)
            .await
    }

    async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.adapter
            .read_by_tag(shard, tag, start_offset, read_config)
            .await
    }

    async fn read_by_key(&self, shard: &str, key: &str) -> Result<Vec<StorageRecord>, CommonError> {
        self.adapter.read_by_key(shard, key).await
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<Option<u64>, CommonError> {
        self.adapter
            .get_offset_by_timestamp(shard, timestamp, strategy)
            .await
    }

    async fn get_offset_by_group(
        &self,
        _group: &str,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, CommonError> {
        Ok(Vec::new())
    }

    async fn commit_offset(
        &self,
        _group_name: &str,
        _offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}
