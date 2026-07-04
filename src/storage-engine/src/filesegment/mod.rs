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

use metadata_struct::storage::segment::{segment_name, EngineSegment};

pub mod delete;
pub mod expire;
pub mod file;
pub mod index;
pub mod read;
pub mod replica;
pub mod scroll;
pub mod write_io_work;
pub mod write_manager;

#[cfg(test)]
mod tests {
    use super::SegmentIdentity;
    use crate::core::test_tool::test_init_segment;
    use crate::filesegment::delete::{delete_by_segment, delete_by_shard};
    use crate::filesegment::file::SegmentFile;
    use crate::filesegment::read::{
        segment_read_by_key, segment_read_by_offset, segment_read_by_tag,
    };
    use crate::filesegment::write_manager::{WriteChannelDataRecord, WriteManager};
    use bytes::Bytes;
    use common_config::storage::StorageType;
    use grpc_clients::pool::ClientPool;
    use rocksdb_engine::rocksdb::RocksDBEngine;
    use std::sync::Arc;
    use tokio::sync::broadcast;
    use tokio::time::{sleep, Duration};

    use crate::core::cache::StorageCacheManager;

    fn make_record(i: u64) -> WriteChannelDataRecord {
        WriteChannelDataRecord {
            pkid: i,
            header: None,
            key: Some(format!("key-{}", i).into()),
            tags: Some(vec!["shared-tag".to_string(), format!("tag-{}", i)]),
            value: Bytes::from(format!("value-{}", i)),
            protocol_data: None,
            expire_at: 0,
        }
    }

    async fn setup_and_write(
        n: u64,
    ) -> (
        SegmentIdentity,
        Arc<StorageCacheManager>,
        String,
        Arc<RocksDBEngine>,
    ) {
        let (segment_iden, cache_manager, fold, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;

        let client_pool = Arc::new(ClientPool::new(100));
        let write_manager =
            WriteManager::new(rocksdb.clone(), cache_manager.clone(), client_pool, 3);

        let (stop_send, _) = broadcast::channel(2);
        write_manager.start(stop_send.clone());
        sleep(Duration::from_millis(100)).await;

        let data_list: Vec<_> = (0..n).map(make_record).collect();
        let resp = write_manager.write(&segment_iden, data_list).await.unwrap();
        assert!(resp.error.is_none());

        stop_send.send(true).ok();
        sleep(Duration::from_millis(100)).await;

        (segment_iden, cache_manager, fold, rocksdb)
    }

    // Write → read_by_offset / read_by_key / read_by_tag → delete_segment → verify gone.
    #[tokio::test]
    async fn filesegment_full_lifecycle() {
        let (seg, cache, fold, db) = setup_and_write(10).await;

        // --- read by offset ---
        let mut sf = SegmentFile::new(seg.shard_name.clone(), seg.segment, fold.clone())
            .await
            .unwrap();
        let results = segment_read_by_offset(&db, &mut sf, &seg, 0, 1 << 30, 100)
            .await
            .unwrap();
        assert_eq!(results.len(), 10, "should read all 10 records by offset");
        // records are in offset order
        for (i, row) in (0u64..).zip(&results) {
            assert_eq!(row.record.metadata.offset, i);
        }

        // --- read by offset with floor-seek (start in the middle) ---
        let partial = segment_read_by_offset(&db, &mut sf, &seg, 7, 1 << 30, 100)
            .await
            .unwrap();
        assert_eq!(partial.len(), 3, "offsets 7..9 = 3 records");
        assert_eq!(partial[0].record.metadata.offset, 7);

        // --- read by key ---
        let by_key = segment_read_by_key(&cache, &db, &seg.shard_name, b"key-5")
            .await
            .unwrap();
        assert_eq!(by_key.len(), 1);
        assert_eq!(by_key[0].record.metadata.offset, 5);

        // --- read by shared tag (all 10 records carry it) ---
        let by_shared = segment_read_by_tag(&cache, &db, &seg.shard_name, "shared-tag", None, 100)
            .await
            .unwrap();
        assert_eq!(by_shared.len(), 10);

        // --- read by unique per-record tag ---
        let by_tag = segment_read_by_tag(&cache, &db, &seg.shard_name, "tag-3", None, 10)
            .await
            .unwrap();
        assert_eq!(by_tag.len(), 1);
        assert_eq!(by_tag[0].record.metadata.offset, 3);

        // --- delete segment (two-step: storage layer + cache layer) ---
        delete_by_segment(&cache, &db, &seg).await.unwrap();
        cache.delete_segment(&seg);

        // key/tag index cleared → reads return empty without touching the file
        let gone_key = segment_read_by_key(&cache, &db, &seg.shard_name, b"key-5")
            .await
            .unwrap();
        assert!(
            gone_key.is_empty(),
            "key index must be cleared after delete"
        );

        let gone_tag = segment_read_by_tag(&cache, &db, &seg.shard_name, "shared-tag", None, 100)
            .await
            .unwrap();
        assert!(
            gone_tag.is_empty(),
            "tag index must be cleared after delete"
        );

        // physical file is gone — verify via exists(), not via read_by_offset:
        // read_by_offset calls ensure_mmap which errors on a missing file rather
        // than returning an empty slice.
        let sf2 = SegmentFile::new(seg.shard_name.clone(), seg.segment, fold)
            .await
            .unwrap();
        assert!(!sf2.exists(), "segment file must be physically deleted");
    }

    // delete_by_shard wipes everything: RocksDB keys + physical directory.
    #[tokio::test]
    async fn filesegment_delete_shard_clears_all() {
        let (seg, cache, _fold, db) = setup_and_write(5).await;

        // Sanity: data is readable before deletion.
        let by_key = segment_read_by_key(&cache, &db, &seg.shard_name, b"key-2")
            .await
            .unwrap();
        assert_eq!(by_key.len(), 1);

        // Delete entire shard (storage + cache).
        delete_by_shard(&cache, &db, &seg.shard_name).await;
        cache.delete_shard(&seg.shard_name);

        // Index gone → key/tag reads return empty.
        let gone = segment_read_by_key(&cache, &db, &seg.shard_name, b"key-2")
            .await
            .unwrap();
        assert!(gone.is_empty(), "key index must be gone after shard delete");

        let gone_tag = segment_read_by_tag(&cache, &db, &seg.shard_name, "shared-tag", None, 100)
            .await
            .unwrap();
        assert!(
            gone_tag.is_empty(),
            "tag index must be gone after shard delete"
        );

        // Note: delete_by_shard removes physical files by iterating
        // broker_config().storage_runtime.data_path. In unit tests that field
        // is empty (default_broker_config sets no data_path), so the directory
        // removal loop is a no-op. The meaningful unit-test invariant is that
        // the RocksDB index is cleared, which the key/tag assertions above
        // already cover.
    }

    // Records written with expire_at in the past must be filtered out at read time.
    #[tokio::test]
    async fn filesegment_expired_records_filtered() {
        let (seg, cache, fold, db) = test_init_segment(StorageType::EngineSegment).await;

        let client_pool = Arc::new(ClientPool::new(100));
        let write_manager = WriteManager::new(db.clone(), cache.clone(), client_pool, 3);

        let (stop_send, _) = broadcast::channel(2);
        write_manager.start(stop_send.clone());
        sleep(Duration::from_millis(100)).await;

        // record 0: already expired (expire_at = 1, in the past)
        // record 1: no expiry
        let data_list = vec![
            WriteChannelDataRecord {
                pkid: 0,
                header: None,
                key: Some("expired-key".into()),
                tags: Some(vec!["ttl-tag".into()]),
                value: Bytes::from("expired"),
                protocol_data: None,
                expire_at: 1,
            },
            WriteChannelDataRecord {
                pkid: 1,
                header: None,
                key: Some("live-key".into()),
                tags: Some(vec!["ttl-tag".into()]),
                value: Bytes::from("live"),
                protocol_data: None,
                expire_at: 0,
            },
        ];
        write_manager.write(&seg, data_list).await.unwrap();
        stop_send.send(true).ok();
        sleep(Duration::from_millis(100)).await;

        // read_by_offset: only the live record survives
        let mut sf = SegmentFile::new(seg.shard_name.clone(), seg.segment, fold)
            .await
            .unwrap();
        let results = segment_read_by_offset(&db, &mut sf, &seg, 0, 1 << 30, 100)
            .await
            .unwrap();
        assert_eq!(results.len(), 1, "expired record must be filtered");
        assert_eq!(
            results[0].record.metadata.key.as_deref(),
            Some(b"live-key".as_ref())
        );

        // read_by_key: expired record is filtered even though the index entry exists
        let res = segment_read_by_key(&cache, &db, &seg.shard_name, b"expired-key")
            .await
            .unwrap();
        assert!(res.is_empty(), "expired record must not be returned by key");

        let res = segment_read_by_key(&cache, &db, &seg.shard_name, b"live-key")
            .await
            .unwrap();
        assert_eq!(res.len(), 1);

        // read_by_tag: both records share "ttl-tag", only the live one survives
        let tag_results = segment_read_by_tag(&cache, &db, &seg.shard_name, "ttl-tag", None, 10)
            .await
            .unwrap();
        assert_eq!(tag_results.len(), 1);
        assert_eq!(
            tag_results[0].record.metadata.key.as_deref(),
            Some(b"live-key".as_ref())
        );
    }
}

/// A unique identifier for a segment, used to get segment metadata or segment file.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Default)]
pub struct SegmentIdentity {
    pub shard_name: String,
    pub segment: u32,
}

impl SegmentIdentity {
    pub fn name(&self) -> String {
        segment_name(&self.shard_name, self.segment)
    }

    pub fn new(shard_name: &str, segment_seq: u32) -> Self {
        SegmentIdentity {
            shard_name: shard_name.to_string(),
            segment: segment_seq,
        }
    }

    pub fn from_journal_segment(segment: &EngineSegment) -> Self {
        SegmentIdentity {
            shard_name: segment.shard_name.to_string(),
            segment: segment.segment_seq,
        }
    }
}
