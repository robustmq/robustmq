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

use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::core::error::StorageEngineError;
use crate::isr::log::ReplicaLog;
use async_trait::async_trait;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::record::StorageRecord;

#[async_trait]
impl ReplicaLog for MemoryStorageEngine {
    async fn append_at(
        &self,
        shard: &str,
        _segment_seq: u32,
        base_offset: u64,
        records: Vec<StorageRecord>,
    ) -> Result<(), StorageEngineError> {
        let shard_state = self.get_or_create_shard(shard);
        let leo = self.commit_log_offset.get_latest_offset(shard)?;
        if base_offset != leo {
            return Err(StorageEngineError::OutOfOrder(
                shard.to_string(),
                base_offset,
                leo,
            ));
        }

        let mut new_leo = leo;
        for record in records {
            new_leo = record.metadata.offset + 1;
            shard_state.data.insert(record.metadata.offset, record);
        }
        self.commit_log_offset.save_latest_offset(shard, new_leo)?;
        Ok(())
    }

    async fn read_from(
        &self,
        shard: &str,
        _segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let read_config = AdapterReadConfig {
            max_record_num: u64::MAX,
            max_size: max_bytes,
        };
        self.read_by_offset(shard, offset, &read_config).await
    }

    fn latest_offset(&self, shard: &str, _segment_seq: u32) -> Result<u64, StorageEngineError> {
        self.commit_log_offset.get_latest_offset(shard)
    }

    async fn truncate_to(
        &self,
        shard: &str,
        _segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let shard_state = self.get_or_create_shard(shard);
        shard_state.data.retain(|&o, _| o <= offset);
        let new_leo = self
            .commit_log_offset
            .get_latest_offset(shard)?
            .min(offset + 1);
        self.commit_log_offset.save_latest_offset(shard, new_leo)?;
        Ok(())
    }

    async fn clear(&self, shard: &str, _segment_seq: u32) -> Result<(), StorageEngineError> {
        let shard_state = self.get_or_create_shard(shard);
        shard_state.data.clear();
        self.commit_log_offset.save_latest_offset(shard, 0)?;
        Ok(())
    }

    fn log_start_offset(&self, shard: &str, _segment_seq: u32) -> Result<u64, StorageEngineError> {
        self.commit_log_offset.get_earliest_offset(shard)
    }

    fn update_high_watermark(&self, shard: &str, hw: u64) -> Result<(), StorageEngineError> {
        self.commit_log_offset
            .save_high_watermark_offset(shard, hw)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::commitlog::memory::engine::MemoryStorageEngine;
    use crate::core::test_tool::test_build_memory_engine;
    use crate::isr::log::ReplicaLog;
    use bytes::Bytes;
    use metadata_struct::storage::record::StorageRecord;

    fn record(offset: u64, data: &str) -> StorageRecord {
        StorageRecord {
            metadata: metadata_struct::storage::record::StorageRecordMetadata {
                offset,
                ..Default::default()
            },
            protocol_data: None,
            data: Bytes::from(data.to_string()),
        }
    }

    fn init_offsets(engine: &MemoryStorageEngine, shard: &str) {
        engine.cache_manager.save_offset_state(
            shard.to_string(),
            crate::core::offset::ShardOffsetState::default(),
        );
    }

    #[tokio::test]
    async fn append_read_roundtrip() {
        let engine = test_build_memory_engine();
        init_offsets(&engine, "s");
        engine
            .append_at(
                "s",
                0,
                0,
                vec![record(0, "a"), record(1, "b"), record(2, "c")],
            )
            .await
            .unwrap();

        assert_eq!(engine.latest_offset("s", 0).unwrap(), 3);
        let read = engine.read_from("s", 0, 1, 1024 * 1024).await.unwrap();
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].data, Bytes::from("b"));
    }

    #[tokio::test]
    async fn append_out_of_order_is_rejected() {
        let engine = test_build_memory_engine();
        init_offsets(&engine, "s");
        engine
            .append_at("s", 0, 0, vec![record(0, "a")])
            .await
            .unwrap();
        // base_offset (5) doesn't match local LEO (1) -> must truncate first
        let err = engine.append_at("s", 0, 5, vec![record(5, "x")]).await;
        assert!(matches!(
            err,
            Err(crate::core::error::StorageEngineError::OutOfOrder(..))
        ));
    }

    #[tokio::test]
    async fn truncate_resets_leo() {
        let engine = test_build_memory_engine();
        init_offsets(&engine, "s");
        engine
            .append_at(
                "s",
                0,
                0,
                vec![
                    record(0, "a"),
                    record(1, "b"),
                    record(2, "c"),
                    record(3, "d"),
                ],
            )
            .await
            .unwrap();
        engine.truncate_to("s", 0, 1).await.unwrap();
        assert_eq!(engine.latest_offset("s", 0).unwrap(), 2);
        let read = engine.read_from("s", 0, 0, 1024 * 1024).await.unwrap();
        assert_eq!(read.len(), 2);
    }

    #[tokio::test]
    async fn clear_empties_log() {
        let engine = test_build_memory_engine();
        init_offsets(&engine, "s");
        engine
            .append_at("s", 0, 0, vec![record(0, "a"), record(1, "b")])
            .await
            .unwrap();
        engine.clear("s", 0).await.unwrap();
        assert_eq!(engine.latest_offset("s", 0).unwrap(), 0);
        let read = engine.read_from("s", 0, 0, 1024 * 1024).await.unwrap();
        assert!(read.is_empty());
    }
}
