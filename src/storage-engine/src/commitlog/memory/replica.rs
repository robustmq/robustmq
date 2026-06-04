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
use metadata_struct::storage::record::StorageRecord;

/// In-memory `ReplicaLog`: nothing is persisted, a restart is a fresh replica (§9.5).
/// Stores into the same `ShardState.data` the producer/consumer path uses, so a
/// record has one identity and a leader switch needs no data move. memory has a
/// single segment, so `segment_seq` is unused.
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
        let leo: u64 = self.leo(shard);
        if base_offset != leo {
            return Err(StorageEngineError::OutOfOrder(
                shard.to_string(),
                base_offset,
                leo,
            ));
        }

        // The leader assigns offsets; each record carries its own and the batch
        // is contiguous, so store by record offset.
        let mut new_leo = leo;
        for record in records {
            new_leo = record.metadata.offset + 1;
            shard_state.data.insert(record.metadata.offset, record);
        }
        self.set_leo(shard, new_leo);
        Ok(())
    }

    async fn read_from(
        &self,
        shard: &str,
        _segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let leo = self.leo(shard);
        if offset > leo {
            return Err(StorageEngineError::OffsetOutOfRange(
                shard.to_string(),
                offset,
                0,
                leo,
            ));
        }
        let Some(shard_state) = self.shards.get(shard) else {
            return Ok(Vec::new());
        };

        let mut records = Vec::new();
        let mut total_bytes = 0u64;
        for current in offset..leo {
            let Some(record) = shard_state.data.get(&current) else {
                continue;
            };
            let size = record.data.len() as u64;
            if !records.is_empty() && total_bytes + size > max_bytes {
                break;
            }
            total_bytes += size;
            records.push(record.clone());
        }
        Ok(records)
    }

    fn latest_offset(&self, shard: &str, _segment_seq: u32) -> Result<u64, StorageEngineError> {
        Ok(self.leo(shard))
    }

    async fn truncate_to(
        &self,
        shard: &str,
        _segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let shard_state = self.get_or_create_shard(shard);
        shard_state.data.retain(|&o, _| o <= offset);
        let new_leo = self.leo(shard).min(offset + 1);
        self.set_leo(shard, new_leo);
        Ok(())
    }

    async fn clear(&self, shard: &str, _segment_seq: u32) -> Result<(), StorageEngineError> {
        let shard_state = self.get_or_create_shard(shard);
        shard_state.data.clear();
        self.set_leo(shard, 0);
        Ok(())
    }

    fn log_start_offset(&self, _shard: &str, _segment_seq: u32) -> Result<u64, StorageEngineError> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
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

    #[tokio::test]
    async fn append_read_roundtrip() {
        let engine = test_build_memory_engine();
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
