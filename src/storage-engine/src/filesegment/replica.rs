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

use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::core::offset::ShardOffset;
use crate::filesegment::file::{data_file_segment, open_segment_write};
use crate::filesegment::index::build::{
    delete_segment_index, save_index, BuildIndexRaw, IndexTypeEnum,
};
use crate::filesegment::read::segment_read_by_offset;
use crate::filesegment::SegmentIdentity;
use crate::isr::log::ReplicaLog;
use async_trait::async_trait;
use metadata_struct::storage::record::StorageRecord;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::io::SeekFrom;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tracing::warn;

/// File-segment implementation of `ReplicaLog`.
///
/// Holds only lightweight handles (`Arc`s and value-type offset trackers).
/// Constructed once per engine instance and shared across ISR tasks via `Arc`.
pub struct FileSegmentReplicaLog {
    cache_manager: Arc<StorageCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    shard_offset: ShardOffset,
}

impl FileSegmentReplicaLog {
    pub fn new(
        cache_manager: Arc<StorageCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        let shard_offset = ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
        FileSegmentReplicaLog {
            cache_manager,
            rocksdb_engine_handler,
            shard_offset,
        }
    }
}

#[async_trait]
impl ReplicaLog for FileSegmentReplicaLog {
    /// Append leader-replicated records to the local segment file.
    ///
    /// The offsets in `records` are set by the leader and must match the local
    /// LEO exactly (`base_offset == leo`).  Returns an error if they don't,
    /// which triggers the truncation flow in the fetcher.
    async fn append_at(
        &self,
        shard: &str,
        segment_seq: u32,
        base_offset: u64,
        records: Vec<StorageRecord>,
    ) -> Result<(), StorageEngineError> {
        if records.is_empty() {
            return Ok(());
        }
        let segment_iden = SegmentIdentity::new(shard, segment_seq);
        let leo = self.shard_offset.get_latest_offset(shard)?;

        if base_offset != leo {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "segment {} out-of-order: expected base_offset={} but got {}",
                segment_iden.name(),
                leo,
                base_offset,
            )));
        }

        let mut segment_file = open_segment_write(&self.cache_manager, &segment_iden).await?;
        let offset_positions = segment_file.write(&records).await?;

        let index_entries: Vec<BuildIndexRaw> = records
            .iter()
            .flat_map(|r| {
                let mut v = vec![BuildIndexRaw {
                    index_type: IndexTypeEnum::Offset,
                    offset: r.metadata.offset,
                    timestamp: Some(r.metadata.create_t),
                    ..Default::default()
                }];
                if let Some(ref key) = r.metadata.key {
                    v.push(BuildIndexRaw {
                        index_type: IndexTypeEnum::Key,
                        key: Some(key.clone()),
                        offset: r.metadata.offset,
                        ..Default::default()
                    });
                }
                if let Some(ref tags) = r.metadata.tags {
                    for tag in tags {
                        v.push(BuildIndexRaw {
                            index_type: IndexTypeEnum::Tag,
                            tag: Some(tag.clone()),
                            offset: r.metadata.offset,
                            ..Default::default()
                        });
                    }
                }
                v
            })
            .collect();

        save_index(
            &self.rocksdb_engine_handler,
            &segment_iden,
            &index_entries,
            &offset_positions,
        )?;

        // Advance LEO = offset of the last record + 1.
        let new_leo = records.last().map(|r| r.metadata.offset + 1).unwrap_or(leo);
        self.shard_offset.save_latest_offset(shard, new_leo)?;

        Ok(())
    }

    async fn read_from(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let segment_iden = SegmentIdentity::new(shard, segment_seq);
        let mut segment_file = open_segment_write(&self.cache_manager, &segment_iden).await?;
        let results = segment_read_by_offset(
            &self.rocksdb_engine_handler,
            &mut segment_file,
            &segment_iden,
            offset,
            max_bytes,
            u64::MAX,
        )
        .await?;
        Ok(results.into_iter().map(|r| r.record).collect())
    }

    fn latest_offset(&self, shard: &str, _segment_seq: u32) -> Result<u64, StorageEngineError> {
        self.shard_offset.get_latest_offset(shard)
    }

    /// Truncate the segment file to keep records with offset ≤ `offset`.
    ///
    /// Scans from position 0 to find the exact byte boundary, then calls
    /// `set_len`.  All index entries for the segment are deleted so reads fall
    /// back to the correct linear scan; entries for surviving records will be
    /// rebuilt by subsequent `append_at` calls.
    async fn truncate_to(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let segment_iden = SegmentIdentity::new(shard, segment_seq);
        let segment_file = open_segment_write(&self.cache_manager, &segment_iden).await?;
        let file_path = data_file_segment(&segment_file.data_fold, segment_file.segment_no);

        let truncate_pos = scan_truncation_pos(&file_path, offset).await?;

        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .open(&file_path)
            .await?;
        file.set_len(truncate_pos).await?;

        if let Err(e) = delete_segment_index(&self.rocksdb_engine_handler, &segment_iden) {
            warn!(
                "truncate_to: index cleanup failed for {}: {}",
                segment_iden.name(),
                e
            );
        }

        // New LEO = offset + 1 (records [start_offset .. offset] survive).
        self.shard_offset.save_latest_offset(shard, offset + 1)?;

        Ok(())
    }

    /// Drop all records in the segment (used when the leader has evicted it).
    async fn clear(&self, shard: &str, segment_seq: u32) -> Result<(), StorageEngineError> {
        let segment_iden = SegmentIdentity::new(shard, segment_seq);
        let segment_file = open_segment_write(&self.cache_manager, &segment_iden).await?;
        let file_path = data_file_segment(&segment_file.data_fold, segment_file.segment_no);

        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .open(&file_path)
            .await?;
        file.set_len(0).await?;

        if let Err(e) = delete_segment_index(&self.rocksdb_engine_handler, &segment_iden) {
            warn!(
                "clear: index cleanup failed for {}: {}",
                segment_iden.name(),
                e
            );
        }

        // Reset LEO to 0 (clearing a segment resets the write pointer).
        self.shard_offset.save_latest_offset(shard, 0)?;

        Ok(())
    }

    fn log_start_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        let segment_iden = SegmentIdentity::new(shard, segment_seq);
        Ok(self
            .cache_manager
            .get_segment_meta(&segment_iden)
            .map(|m| m.start_offset.max(0) as u64)
            .unwrap_or(0))
    }

    /// Update the follower's local HW.
    ///
    /// Written to `CommitLogOffset` (same key the leader uses for acks=all
    /// waits) so that the HW broadcast wakes any pending `wait_for_hw` futures.
    fn update_high_watermark(&self, shard: &str, hw: u64) -> Result<(), StorageEngineError> {
        self.shard_offset
            .save_high_watermark_offset(shard, hw)
            .map(|_| ())
    }
}

/// Scan the segment file sequentially to find the byte offset of the first
/// record whose `metadata.offset > truncate_offset`.
///
/// On-disk record layout (big-endian):
/// ```text
/// offset(u64,8) | total_len(u32,4) | metadata_len(u32,4) | metadata(?)
/// | protocol_data_len(u32,4) | protocol_data(?) | data_len(u32,4) | data(?)
/// ```
/// Total record bytes = 24 + total_len.
///
/// Returns the byte position to pass to `set_len`, i.e., the position right
/// after the last record to keep.  Returns 0 if no record satisfies the
/// predicate (truncate the whole file).
async fn scan_truncation_pos(
    file_path: &str,
    truncate_offset: u64,
) -> Result<u64, StorageEngineError> {
    let file = match tokio::fs::File::open(file_path).await {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(e.into()),
    };

    let mut reader = tokio::io::BufReader::new(file);
    let mut pos: u64 = 0;
    let mut keep_until: u64 = 0;

    loop {
        let record_offset = match reader.read_u64().await {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        };
        let total_len = match reader.read_u32().await {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        };
        // 8 (offset) + 4 (total_len) + 4 (metadata_len) + metadata + 4 (proto_len) + proto + 4 (data_len) + data
        // = 24 + total_len
        let record_size = 24u64 + total_len as u64;

        if record_offset <= truncate_offset {
            keep_until = pos + record_size;
            pos = keep_until;
            reader.seek(SeekFrom::Start(pos)).await?;
        } else {
            break;
        }
    }

    Ok(keep_until)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_init_segment;
    use bytes::Bytes;
    use common_config::storage::StorageType;
    use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};

    fn make_record(offset: u64, data: &str) -> StorageRecord {
        StorageRecord {
            metadata: StorageRecordMetadata {
                offset,
                shard: "s".to_string(),
                segment: 0,
                ..Default::default()
            },
            protocol_data: None,
            data: Bytes::from(data.to_string()),
        }
    }

    #[tokio::test]
    async fn append_read_roundtrip() {
        let (segment_iden, cache_manager, _fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;
        let shard = segment_iden.shard_name.clone();
        let seq = segment_iden.segment;

        let log = FileSegmentReplicaLog::new(cache_manager, rocksdb);
        log.append_at(
            &shard,
            seq,
            0,
            vec![make_record(0, "a"), make_record(1, "b")],
        )
        .await
        .unwrap();

        assert_eq!(log.latest_offset(&shard, seq).unwrap(), 2);

        let recs = log.read_from(&shard, seq, 0, 1024 * 1024).await.unwrap();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].data, Bytes::from("a"));
        assert_eq!(recs[1].data, Bytes::from("b"));
    }

    #[tokio::test]
    async fn out_of_order_rejected() {
        let (segment_iden, cache_manager, _fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;
        let shard = segment_iden.shard_name.clone();
        let seq = segment_iden.segment;

        let log = FileSegmentReplicaLog::new(cache_manager, rocksdb);
        log.append_at(&shard, seq, 0, vec![make_record(0, "a")])
            .await
            .unwrap();

        // base_offset (5) != leo (1) — must return an error
        assert!(log
            .append_at(&shard, seq, 5, vec![make_record(5, "x")])
            .await
            .is_err());
    }

    #[tokio::test]
    async fn truncate_drops_tail() {
        let (segment_iden, cache_manager, _fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;
        let shard = segment_iden.shard_name.clone();
        let seq = segment_iden.segment;

        let log = FileSegmentReplicaLog::new(cache_manager, rocksdb);
        log.append_at(
            &shard,
            seq,
            0,
            vec![
                make_record(0, "a"),
                make_record(1, "b"),
                make_record(2, "c"),
            ],
        )
        .await
        .unwrap();

        log.truncate_to(&shard, seq, 0).await.unwrap();
        assert_eq!(log.latest_offset(&shard, seq).unwrap(), 1);

        let recs = log.read_from(&shard, seq, 0, 1024 * 1024).await.unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].data, Bytes::from("a"));
    }

    #[tokio::test]
    async fn clear_empties_segment() {
        let (segment_iden, cache_manager, _fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;
        let shard = segment_iden.shard_name.clone();
        let seq = segment_iden.segment;

        let log = FileSegmentReplicaLog::new(cache_manager, rocksdb);
        log.append_at(
            &shard,
            seq,
            0,
            vec![make_record(0, "a"), make_record(1, "b")],
        )
        .await
        .unwrap();
        log.clear(&shard, seq).await.unwrap();

        let recs = log.read_from(&shard, seq, 0, 1024 * 1024).await.unwrap();
        assert!(recs.is_empty());
    }
}
