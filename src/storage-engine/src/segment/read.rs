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

use super::file::SegmentFile;
use super::SegmentIdentity;
use crate::segment::index::offset::OffsetIndexManager;
use crate::segment::index::tag::TagIndexManager;
use crate::{core::error::StorageEngineError, segment::file::ReadData};
use protocol::storage::protocol::{ReadReqFilter, ReadReqOptions};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

/// handle read requests by offset
///
/// Use index (if there's any) to find the last nearest start byte position given the offset
pub async fn read_by_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    filter: &ReadReqFilter,
    read_options: &ReadReqOptions,
) -> Result<Vec<ReadData>, StorageEngineError> {
    let offset_index = OffsetIndexManager::new(rocksdb_engine_handler.clone());
    let start_position = if let Some(position) = offset_index
        .get_last_nearest_position_by_offset(segment_iden, filter.offset)
        .await?
    {
        position.position
    } else {
        0
    };

    let res = segment_file
        .read_by_offset(
            start_position,
            filter.offset,
            read_options.max_size,
            read_options.max_record,
        )
        .await?;

    Ok(res)
}

/// handle read requests by key
///
/// Use index (if there's any) to find all start byte positions of the records with the given key
pub async fn read_by_key(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    filter: &ReadReqFilter,
    read_options: &ReadReqOptions,
) -> Result<Vec<ReadData>, StorageEngineError> {
    let tag_index = TagIndexManager::new(rocksdb_engine_handler.clone());
    let index_data_list = tag_index
        .get_last_positions_by_key(
            segment_iden,
            filter.offset,
            filter.key.clone(),
            read_options.max_record,
        )
        .await?;

    let positions = index_data_list.iter().map(|raw| raw.position).collect();

    segment_file.read_by_positions(positions).await
}

/// handle read requests by tag
///
/// Similar to [`read_by_key`], but use tag index
pub async fn read_by_tag(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    filter: &ReadReqFilter,
    read_options: &ReadReqOptions,
) -> Result<Vec<ReadData>, StorageEngineError> {
    let tag_index = TagIndexManager::new(rocksdb_engine_handler.clone());
    let index_data_list = tag_index
        .get_last_positions_by_tag(
            segment_iden,
            filter.offset,
            filter.tag.clone(),
            read_options.max_record,
        )
        .await?;
    let positions = index_data_list.iter().map(|raw| raw.position).collect();
    segment_file.read_by_positions(positions).await
}

#[cfg(test)]
mod tests {
    use super::{read_by_key, read_by_offset, read_by_tag};
    use crate::core::test::test_write_and_build_index;
    use crate::segment::file::SegmentFile;
    use protocol::storage::protocol::{ReadReqFilter, ReadReqOptions};

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn read_by_offset_test() {
        let (segment_iden, _, _, fold, rocksdb_engine_handler) =
            test_write_and_build_index(30, 5).await;

        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold);

        let read_options = ReadReqOptions {
            max_record: 2,
            max_size: 1024 * 1024 * 1024,
        };

        let filter = ReadReqFilter {
            offset: 5,
            ..Default::default()
        };
        let res = read_by_offset(
            &rocksdb_engine_handler,
            &segment_file,
            &segment_iden,
            &filter,
            &read_options,
        )
        .await;
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert_eq!(resp.len(), 2);

        let mut i = 5;
        for row in resp {
            println!("{row:?}");
            assert_eq!(row.record.metadata.key.unwrap(), format!("key-{i}"));
            i += 1;
        }

        let read_options = ReadReqOptions {
            max_record: 5,
            max_size: 1024 * 1024 * 1024,
        };
        let filter = ReadReqFilter {
            offset: 10,
            ..Default::default()
        };
        let res = read_by_offset(
            &rocksdb_engine_handler,
            &segment_file,
            &segment_iden,
            &filter,
            &read_options,
        )
        .await;
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert_eq!(resp.len(), 5);

        let mut i = 10;
        for row in resp {
            assert_eq!(row.record.metadata.key.unwrap(), format!("key-{i}"));
            i += 1;
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn read_by_key_test() {
        let (segment_iden, _, _, fold, rocksdb_engine_handler) =
            test_write_and_build_index(30, 10).await;

        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold);

        let read_options = ReadReqOptions {
            max_record: 10,
            max_size: 1024 * 1024 * 1024,
        };

        let key = "key-5".to_string();
        let filter = ReadReqFilter {
            key: key.clone(),
            offset: 0,
            ..Default::default()
        };
        let res = read_by_key(
            &rocksdb_engine_handler,
            &segment_file,
            &segment_iden,
            &filter,
            &read_options,
        )
        .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert_eq!(resp.len(), 1);
        let meata = resp.first().unwrap().record.metadata.clone();
        assert_eq!(meata.key.unwrap(), key);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn read_by_tag_test() {
        let (segment_iden, _, _, fold, rocksdb_engine_handler) =
            test_write_and_build_index(30, 10).await;

        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold);

        let read_options = ReadReqOptions {
            max_record: 10,
            max_size: 1024 * 1024 * 1024,
        };

        let tag = "tag-5".to_string();
        let filter = ReadReqFilter {
            tag: tag.clone(),
            offset: 0,
            ..Default::default()
        };
        let res = read_by_tag(
            &rocksdb_engine_handler,
            &segment_file,
            &segment_iden,
            &filter,
            &read_options,
        )
        .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert_eq!(resp.len(), 1);

        let meata = resp.first().unwrap().record.metadata.clone();
        assert!(meata.tags.unwrap().contains(&tag));
    }
}
