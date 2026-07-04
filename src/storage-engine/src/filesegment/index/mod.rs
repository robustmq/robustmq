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

pub mod build;
pub mod read;

use crate::core::error::StorageEngineError;
use common_base::error::common::CommonError;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_STORAGE_ENGINE;
use std::sync::Arc;

pub(super) fn get_storage_cf(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<Arc<rocksdb::BoundColumnFamily<'_>>, StorageEngineError> {
    rocksdb_engine_handler
        .cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE)
        .ok_or_else(|| {
            CommonError::RocksDBFamilyNotAvailable(DB_COLUMN_FAMILY_STORAGE_ENGINE.to_string())
                .into()
        })
}

#[cfg(test)]
mod tests {
    use super::build::{
        delete_segment_index, delete_shard_index_for_segment, save_index, BuildIndexRaw,
        IndexTypeEnum,
    };
    use super::read::{
        get_index_data_by_key, get_index_data_by_offset, get_index_data_by_tag,
        get_index_data_by_timestamp,
    };
    use crate::core::test_tool::test_build_segment;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::collections::HashMap;

    fn offset_positions() -> HashMap<u64, u64> {
        [
            (0, 100),
            (100, 1000),
            (200, 2000),
            (300, 3000),
            (400, 4000),
            (10000, 50000),
            (20000, 150000),
        ]
        .into_iter()
        .collect()
    }

    fn all_index_data() -> Vec<BuildIndexRaw> {
        vec![
            // Offset
            BuildIndexRaw {
                index_type: IndexTypeEnum::Offset,
                offset: 0,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Offset,
                offset: 10000,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Offset,
                offset: 20000,
                ..Default::default()
            },
            // Key
            BuildIndexRaw {
                index_type: IndexTypeEnum::Key,
                key: Some("user-123".into()),
                offset: 100,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Key,
                key: Some("order-456".into()),
                offset: 200,
                ..Default::default()
            },
            // Tag
            BuildIndexRaw {
                index_type: IndexTypeEnum::Tag,
                tag: Some("urgent".into()),
                offset: 100,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Tag,
                tag: Some("urgent".into()),
                offset: 200,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Tag,
                tag: Some("normal".into()),
                offset: 300,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Tag,
                tag: Some("urgent".into()),
                offset: 400,
                ..Default::default()
            },
            // Time
            BuildIndexRaw {
                index_type: IndexTypeEnum::Time,
                timestamp: Some(1000),
                offset: 0,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Time,
                timestamp: Some(2000),
                offset: 10000,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Time,
                timestamp: Some(3000),
                offset: 20000,
                ..Default::default()
            },
        ]
    }

    #[test]
    fn save_and_read_all_index_types() {
        let rocksdb = test_rocksdb_instance();
        let segment_iden = test_build_segment();
        save_index(
            &rocksdb,
            &segment_iden,
            &all_index_data(),
            &offset_positions(),
        )
        .unwrap();

        // Offset: exact hit and floor-seek
        let data = get_index_data_by_offset(&rocksdb, &segment_iden, 0)
            .unwrap()
            .unwrap();
        assert_eq!((data.offset, data.position), (0, 100));
        let data = get_index_data_by_offset(&rocksdb, &segment_iden, 15000)
            .unwrap()
            .unwrap();
        assert_eq!((data.offset, data.position), (10000, 50000));

        // Key: found and not-found
        let data = get_index_data_by_key(&rocksdb, &segment_iden.shard_name, b"user-123")
            .unwrap()
            .unwrap();
        assert_eq!((data.offset, data.position), (100, 1000));
        assert!(
            get_index_data_by_key(&rocksdb, &segment_iden.shard_name, b"not-exist")
                .unwrap()
                .is_none()
        );

        // Tag: multi-result and offset filter
        let results =
            get_index_data_by_tag(&rocksdb, &segment_iden.shard_name, Some(0), "urgent", 10)
                .unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!((results[0].offset, results[0].position), (100, 1000));
        assert_eq!((results[2].offset, results[2].position), (400, 4000));
        let results =
            get_index_data_by_tag(&rocksdb, &segment_iden.shard_name, Some(150), "urgent", 10)
                .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].offset, 200);

        // Time: exact hit and floor-seek
        let data = get_index_data_by_timestamp(&rocksdb, &segment_iden, 1000)
            .unwrap()
            .unwrap();
        assert_eq!((data.offset, data.position, data.timestamp), (0, 100, 1000));
        let data = get_index_data_by_timestamp(&rocksdb, &segment_iden, 2500)
            .unwrap()
            .unwrap();
        assert_eq!(
            (data.offset, data.position, data.timestamp),
            (10000, 50000, 2000)
        );
    }

    #[test]
    fn delete_segment_index_removes_all_entries() {
        let rocksdb = test_rocksdb_instance();
        let segment_iden = test_build_segment();
        save_index(
            &rocksdb,
            &segment_iden,
            &all_index_data(),
            &offset_positions(),
        )
        .unwrap();

        delete_segment_index(&rocksdb, &segment_iden).unwrap();

        assert!(get_index_data_by_offset(&rocksdb, &segment_iden, 0)
            .unwrap()
            .is_none());
        assert!(get_index_data_by_timestamp(&rocksdb, &segment_iden, 1000)
            .unwrap()
            .is_none());
    }

    #[test]
    fn delete_shard_index_removes_key_and_tag_entries() {
        let rocksdb = test_rocksdb_instance();
        let segment_iden = test_build_segment();
        save_index(
            &rocksdb,
            &segment_iden,
            &all_index_data(),
            &offset_positions(),
        )
        .unwrap();

        delete_shard_index_for_segment(&rocksdb, &segment_iden.shard_name, segment_iden.segment)
            .unwrap();

        // key and tag entries gone
        assert!(
            get_index_data_by_key(&rocksdb, &segment_iden.shard_name, b"user-123")
                .unwrap()
                .is_none()
        );
        assert!(
            get_index_data_by_tag(&rocksdb, &segment_iden.shard_name, Some(0), "urgent", 10)
                .unwrap()
                .is_empty()
        );

        // offset and time entries unaffected (different key prefix)
        assert!(get_index_data_by_offset(&rocksdb, &segment_iden, 0)
            .unwrap()
            .is_some());
        assert!(get_index_data_by_timestamp(&rocksdb, &segment_iden, 1000)
            .unwrap()
            .is_some());
    }

    // seek_for_prev must return None when the target is before all stored entries.
    // This guards the prefix boundary check in get_index_data_by_offset /
    // get_index_data_by_timestamp — without it seek_for_prev would land on the
    // last key of a different prefix and the caller would silently return wrong data.
    #[test]
    fn seek_for_prev_returns_none_before_first_entry() {
        let rocksdb = test_rocksdb_instance();
        let segment_iden = test_build_segment();
        // Smallest indexed offset is 0, smallest timestamp is 1000.
        // Querying for offset u64::MAX on an *empty* segment exercises the "valid()
        // is false" branch; querying a value that only precedes the prefix exercises
        // the "starts_with prefix" guard. Use a brand-new segment (no data at all).
        assert!(
            get_index_data_by_offset(&rocksdb, &segment_iden, 0)
                .unwrap()
                .is_none(),
            "offset seek on empty segment must return None"
        );
        assert!(
            get_index_data_by_timestamp(&rocksdb, &segment_iden, 1)
                .unwrap()
                .is_none(),
            "timestamp seek on empty segment must return None"
        );

        // Now write data starting at offset 100 / timestamp 5000 and verify that
        // a query for offset 50 / timestamp 4999 (before the first entry) still
        // returns None (exercises the prefix boundary guard on a non-empty DB).
        let positions: std::collections::HashMap<u64, u64> =
            [(100, 999), (200, 1999)].into_iter().collect();
        let data = vec![
            BuildIndexRaw {
                index_type: IndexTypeEnum::Offset,
                offset: 100,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Time,
                timestamp: Some(5000),
                offset: 100,
                ..Default::default()
            },
        ];
        save_index(&rocksdb, &segment_iden, &data, &positions).unwrap();

        assert!(
            get_index_data_by_offset(&rocksdb, &segment_iden, 50)
                .unwrap()
                .is_none(),
            "offset 50 is before the first indexed offset 100 — must return None"
        );
        assert!(
            get_index_data_by_timestamp(&rocksdb, &segment_iden, 4999)
                .unwrap()
                .is_none(),
            "timestamp 4999 is before the first indexed timestamp 5000 — must return None"
        );
    }

    // delete_shard_index_for_segment must remove only entries that belong to the
    // requested segment_seq.  Entries from a different segment on the same shard
    // must survive untouched.
    #[test]
    fn delete_shard_index_only_removes_target_segment() {
        use crate::filesegment::SegmentIdentity;

        let rocksdb = test_rocksdb_instance();
        let seg0 = test_build_segment(); // segment = 0
        let seg1 = SegmentIdentity {
            shard_name: seg0.shard_name.clone(),
            segment: 1,
        };

        // segment 0 entries
        let pos0: std::collections::HashMap<u64, u64> =
            [(100, 1000), (200, 2000)].into_iter().collect();
        let data0 = vec![
            BuildIndexRaw {
                index_type: IndexTypeEnum::Key,
                key: Some("seg0-key".into()),
                offset: 100,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Tag,
                tag: Some("shared-tag".into()),
                offset: 100,
                ..Default::default()
            },
        ];
        save_index(&rocksdb, &seg0, &data0, &pos0).unwrap();

        // segment 1 entries (different offsets so keys don't collide)
        let pos1: std::collections::HashMap<u64, u64> =
            [(300, 3000), (400, 4000)].into_iter().collect();
        let data1 = vec![
            BuildIndexRaw {
                index_type: IndexTypeEnum::Key,
                key: Some("seg1-key".into()),
                offset: 300,
                ..Default::default()
            },
            BuildIndexRaw {
                index_type: IndexTypeEnum::Tag,
                tag: Some("shared-tag".into()),
                offset: 300,
                ..Default::default()
            },
        ];
        save_index(&rocksdb, &seg1, &data1, &pos1).unwrap();

        // Delete shard index for segment 0 only.
        delete_shard_index_for_segment(&rocksdb, &seg0.shard_name, seg0.segment).unwrap();

        // segment 0 key/tag entries must be gone.
        assert!(
            get_index_data_by_key(&rocksdb, &seg0.shard_name, b"seg0-key")
                .unwrap()
                .is_none(),
            "seg0 key entry must be deleted"
        );

        // segment 1 entries must survive.
        assert!(
            get_index_data_by_key(&rocksdb, &seg1.shard_name, b"seg1-key")
                .unwrap()
                .is_some(),
            "seg1 key entry must survive"
        );
        let tag_results =
            get_index_data_by_tag(&rocksdb, &seg0.shard_name, Some(0), "shared-tag", 10).unwrap();
        assert_eq!(tag_results.len(), 1, "only seg1 tag entry should remain");
        assert_eq!(
            tag_results[0].segment, 1,
            "remaining tag entry must belong to segment 1"
        );
    }

    // record_num acts as a hard cap: if the tag has N entries and we ask for k < N,
    // exactly k results must come back.
    #[test]
    fn get_index_data_by_tag_respects_record_num_limit() {
        let rocksdb = test_rocksdb_instance();
        let segment_iden = test_build_segment();
        save_index(
            &rocksdb,
            &segment_iden,
            &all_index_data(),
            &offset_positions(),
        )
        .unwrap();

        // "urgent" has 3 entries (offsets 100, 200, 400).  Ask for at most 2.
        let results =
            get_index_data_by_tag(&rocksdb, &segment_iden.shard_name, Some(0), "urgent", 2)
                .unwrap();
        assert_eq!(results.len(), 2, "record_num=2 must cap results at 2");
        assert_eq!(results[0].offset, 100);
        assert_eq!(results[1].offset, 200);
    }
}
