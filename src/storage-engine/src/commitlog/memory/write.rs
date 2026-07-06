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

use crate::{
    commitlog::memory::engine::{MemoryShardData, MemoryStorageEngine},
    core::error::StorageEngineError,
};
use metadata_struct::storage::{
    adapter_read_config::AdapterWriteRespRow, adapter_record::AdapterWriteRecord,
    convert::convert_adapter_record_to_storage,
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
            return Err(StorageEngineError::CommonErrorStr(format!(
                "Write to shard [{}] returned empty result",
                shard
            )));
        }

        Ok(results.first().unwrap().clone())
    }

    pub async fn delete_by_key(&self, shard: &str, key: &[u8]) -> Result<(), StorageEngineError> {
        let offset = {
            let Some(shard_state) = self.shards.get(shard) else {
                return Ok(());
            };
            let Some(offset_ref) = shard_state.key_index.get(key) else {
                return Ok(());
            };
            *offset_ref
        };
        self.delete_by_offset(shard, offset).await
    }

    pub async fn delete_by_offset(
        &self,
        shard: &str,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let Some(shard_state) = self.shards.get(shard) else {
            return Ok(());
        };

        let Some((_, record)) = shard_state.data.remove(&offset) else {
            return Ok(());
        };

        if let Some(key) = &record.metadata.key {
            shard_state.key_index.remove(key);
        }

        if let Some(tags) = &record.metadata.tags {
            for tag in tags.iter() {
                if let Some(mut offsets) = shard_state.tag_index.get_mut(tag) {
                    offsets.retain(|&o| o != offset);
                }
            }
        }

        if record.metadata.create_t > 0 && offset.is_multiple_of(5000) {
            shard_state
                .timestamp_index
                .remove(&record.metadata.create_t);
        }

        Ok(())
    }

    async fn internal_batch_write(
        &self,
        shard_name: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let shard = self.get_or_create_shard(shard_name);
        let _guard = shard.write_lock.lock().await;

        self.key_compaction(&shard, messages);

        let mut offset_res = Vec::with_capacity(messages.len());
        let mut index_entries = Vec::with_capacity(messages.len());
        let mut offset = self.commit_log_offset.get_latest_offset(shard_name)?;

        for msg in messages.iter() {
            offset_res.push(AdapterWriteRespRow {
                pkid: msg.record_id,
                offset,
                ..Default::default()
            });

            let engine_record = convert_adapter_record_to_storage(msg.clone(), shard_name, offset);
            shard.data.insert(offset, engine_record);
            index_entries.push((offset, msg));

            offset += 1;
        }

        MemoryStorageEngine::batch_save_index(&shard, &index_entries);

        self.commit_log_offset
            .save_latest_offset(shard_name, offset)?;
        Ok(offset_res)
    }

    fn key_compaction(&self, shard: &Arc<MemoryShardData>, messages: &[AdapterWriteRecord]) {
        for record in messages.iter() {
            let Some(key) = record.key.as_deref() else {
                continue;
            };
            let Some((_, offset)) = shard.key_index.remove(key) else {
                continue;
            };
            let Some((_, removed)) = shard.data.remove(&offset) else {
                continue;
            };
            if let Some(tags) = &removed.metadata.tags {
                for tag in tags.iter() {
                    if let Some(mut offsets) = shard.tag_index.get_mut(tag) {
                        offsets.retain(|&o| o != offset);
                    }
                }
            }
            if removed.metadata.create_t > 0 && offset.is_multiple_of(5000) {
                shard.timestamp_index.remove(&removed.metadata.create_t);
            }
        }
    }
}
