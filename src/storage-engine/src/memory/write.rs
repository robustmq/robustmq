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

use dashmap::DashMap;
use metadata_struct::storage::{
    adapter_read_config::AdapterWriteRespRow, adapter_record::AdapterWriteRecord,
    convert::convert_adapter_record_to_engine,
};

use crate::{
    core::error::StorageEngineError,
    memory::engine::{MemoryStorageEngine, ShardState},
};

impl MemoryStorageEngine {
    pub async fn batch_write(
        &self,
        shard: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        self.internal_batch_write(shard, messages).await
    }

    pub async fn write(
        &self,
        shard: &str,
        data: &AdapterWriteRecord,
    ) -> Result<AdapterWriteRespRow, StorageEngineError> {
        let results = self
            .internal_batch_write(shard, std::slice::from_ref(data))
            .await?;

        if results.is_empty() {
            return Err(StorageEngineError::CommonErrorStr("".to_string()));
        }

        Ok(results.first().unwrap().clone())
    }

    async fn internal_batch_write(
        &self,
        shard_name: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        // lock
        let lock = self
            .shard_write_locks
            .entry(shard_name.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone();

        let _guard = lock.lock().await;

        // init
        let shard_name_str = shard_name.to_string();
        let current_shard_data_list = self
            .shard_data
            .entry(shard_name_str.clone())
            .or_insert(DashMap::with_capacity(2));

        // validator
        if current_shard_data_list.len() > self.config.max_records_per_shard {
            //todo
        }

        // save data
        let mut offset_res = Vec::with_capacity(messages.len());
        let mut offset = self.get_latest_offset(shard_name)?;
        for msg in messages.iter() {
            offset_res.push(AdapterWriteRespRow {
                pkid: msg.pkid,
                offset,
                ..Default::default()
            });

            let engine_record = convert_adapter_record_to_engine(msg.clone(), shard_name, offset);

            current_shard_data_list.insert(offset, engine_record);

            self.save_index(shard_name, offset, msg);

            offset += 1;
        }

        self.shard_state.insert(
            shard_name.to_string(),
            ShardState {
                earliest_offset: 0,
                latest_offset: offset,
            },
        );
        Ok(offset_res)
    }
}
