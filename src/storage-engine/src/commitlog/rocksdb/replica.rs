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

use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
use crate::core::error::StorageEngineError;
use crate::isr::log::ReplicaLog;
use async_trait::async_trait;
use common_base::utils::serialize::{deserialize, serialize};
use metadata_struct::storage::record::StorageRecord;
use metadata_struct::storage::segment::segment_name;
use rocksdb::WriteBatch;
use rocksdb_engine::keys::storage::{
    shard_record_key, shard_record_key_prefix, shard_segment_leo_key,
};

#[async_trait]
impl ReplicaLog for RocksDBStorageEngine {
    async fn append_at(
        &self,
        shard: &str,
        segment_seq: u32,
        base_offset: u64,
        records: Vec<StorageRecord>,
    ) -> Result<(), StorageEngineError> {
        let cf = self.get_cf()?;
        let leo = self.read_leo(shard, segment_seq)?;

        if base_offset != leo {
            return Err(StorageEngineError::OutOfOrder(
                segment_name(shard, segment_seq),
                base_offset,
                leo,
            ));
        }

        let mut batch = WriteBatch::default();
        let mut new_leo = leo;
        for record in &records {
            let key = shard_record_key(shard, segment_seq, record.metadata.offset);
            batch.put_cf(&cf, key.as_bytes(), serialize(record)?);
            new_leo = record.metadata.offset + 1;
        }
        let leo_key = shard_segment_leo_key(shard, segment_seq);
        batch.put_cf(&cf, leo_key.as_bytes(), serialize(&new_leo)?);

        self.rocksdb_engine_handler.write_batch(batch)?;
        Ok(())
    }

    async fn read_from(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let leo = self.read_leo(shard, segment_seq)?;
        if offset > leo {
            return Err(StorageEngineError::OffsetOutOfRange(
                segment_name(shard, segment_seq),
                offset,
                0,
                leo,
            ));
        }

        let cf = self.get_cf()?;
        let prefix = shard_record_key_prefix(shard, segment_seq);
        let seek_key = shard_record_key(shard, segment_seq, offset);
        let until_key = shard_record_key(shard, segment_seq, leo);

        let mut records = Vec::new();
        let mut total_bytes = 0u64;
        for (_, value) in self
            .rocksdb_engine_handler
            .read_prefix_from(cf, &prefix, &seek_key, &until_key)?
        {
            let record: StorageRecord = deserialize(&value)?;
            let size = record.data.len() as u64;
            if !records.is_empty() && total_bytes + size > max_bytes {
                break;
            }
            total_bytes += size;
            records.push(record);
        }
        Ok(records)
    }

    fn latest_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        self.read_leo(shard, segment_seq)
    }

    async fn truncate_to(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let cf = self.get_cf()?;
        let new_leo = self.read_leo(shard, segment_seq)?.min(offset + 1);

        let leo_key = shard_segment_leo_key(shard, segment_seq);
        self.rocksdb_engine_handler
            .write(cf.clone(), &leo_key, &new_leo)?;

        let prefix = shard_record_key_prefix(shard, segment_seq);
        let from = shard_record_key(shard, segment_seq, offset + 1);
        let to = self.rocksdb_engine_handler.prefix_range_end(&prefix);
        self.rocksdb_engine_handler
            .delete_range_cf(cf, from.into_bytes(), to)?;
        Ok(())
    }

    async fn clear(&self, shard: &str, segment_seq: u32) -> Result<(), StorageEngineError> {
        let cf = self.get_cf()?;
        let leo_key = shard_segment_leo_key(shard, segment_seq);
        self.rocksdb_engine_handler
            .write(cf.clone(), &leo_key, &0u64)?;

        let prefix = shard_record_key_prefix(shard, segment_seq);
        self.rocksdb_engine_handler.delete_prefix(cf, &prefix)?;
        Ok(())
    }

    fn log_start_offset(&self, shard: &str, _segment_seq: u32) -> Result<u64, StorageEngineError> {
        self.commitlog_offset.get_earliest_offset(shard)
    }

    fn commit_log_offset(&self) -> &crate::commitlog::offset::CommitLogOffset {
        &self.commitlog_offset
    }
}

impl RocksDBStorageEngine {
    fn read_leo(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        let cf = self.get_cf()?;
        let key = shard_segment_leo_key(shard, segment_seq);
        Ok(self
            .rocksdb_engine_handler
            .read::<u64>(cf, &key)?
            .unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::error::StorageEngineError;
    use crate::core::test_tool::test_build_rocksdb_engine;
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
        let engine = test_build_rocksdb_engine();
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
        let engine = test_build_rocksdb_engine();
        engine
            .append_at("s", 0, 0, vec![record(0, "a")])
            .await
            .unwrap();
        // base_offset (5) doesn't match local LEO (1) -> must truncate first
        assert!(matches!(
            engine.append_at("s", 0, 5, vec![record(5, "x")]).await,
            Err(StorageEngineError::OutOfOrder(..))
        ));
    }

    #[tokio::test]
    async fn truncate_resets_leo_and_drops_tail() {
        let engine = test_build_rocksdb_engine();
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
        let engine = test_build_rocksdb_engine();
        engine
            .append_at("s", 0, 0, vec![record(0, "a"), record(1, "b")])
            .await
            .unwrap();
        engine.clear("s", 0).await.unwrap();
        assert_eq!(engine.latest_offset("s", 0).unwrap(), 0);
        assert!(engine
            .read_from("s", 0, 0, 1024 * 1024)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn append_after_truncate_continues() {
        let engine = test_build_rocksdb_engine();
        engine
            .append_at(
                "s",
                0,
                0,
                vec![record(0, "a"), record(1, "b"), record(2, "c")],
            )
            .await
            .unwrap();
        engine.truncate_to("s", 0, 0).await.unwrap();
        assert_eq!(engine.latest_offset("s", 0).unwrap(), 1);
        engine
            .append_at("s", 0, 1, vec![record(1, "z")])
            .await
            .unwrap();
        let read = engine.read_from("s", 0, 0, 1024 * 1024).await.unwrap();
        assert_eq!(read.len(), 2);
        assert_eq!(read[1].data, Bytes::from("z"));
    }
}
