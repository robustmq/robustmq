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
use crate::{
    core::error::StorageEngineError,
    segment::{
        file::ReadData,
        index::read::{get_index_data_by_key, get_index_data_by_offset, get_index_data_by_tag},
    },
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

/// handle read requests by offset
///
/// Use index (if there's any) to find the last nearest start byte position given the offset
pub async fn segment_read_by_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    shard_name: &str,
    offset: u64,
    max_size: u64,
    max_record: u64,
) -> Result<Vec<ReadData>, StorageEngineError> {
    let start_position = if let Some(position) =
        get_index_data_by_offset(rocksdb_engine_handler, shard_name, offset)?
    {
        position.position
    } else {
        0
    };

    let res = segment_file
        .read_by_offset(start_position, offset, max_size, max_record)
        .await?;
    Ok(res)
}

/// handle read requests by key
///
/// Use index (if there's any) to find all start byte positions of the records with the given key
pub async fn segment_read_by_key(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    shard_name: &str,
    key: &str,
) -> Result<Vec<ReadData>, StorageEngineError> {
    let index_data = get_index_data_by_key(rocksdb_engine_handler, shard_name, key.to_string())?;

    let positions = if let Some(index) = index_data {
        vec![index.position]
    } else {
        return Ok(Vec::new());
    };

    segment_file.read_by_positions(positions).await
}

/// handle read requests by tag
///
/// Similar to [`read_by_key`], but use tag index
pub async fn segment_read_by_tag(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    shard_name: &str,
    tag: &str,
    start_offset: Option<u64>,
    max_record: u64,
) -> Result<Vec<ReadData>, StorageEngineError> {
    let index_data_list = get_index_data_by_tag(
        rocksdb_engine_handler,
        shard_name,
        start_offset,
        tag,
        max_record as usize,
    )?;
    let positions = index_data_list.iter().map(|raw| raw.position).collect();
    segment_file.read_by_positions(positions).await
}

#[cfg(test)]
mod tests {
    use super::{segment_read_by_key, segment_read_by_offset, segment_read_by_tag};
    use crate::{core::test::test_base_write_data, segment::file::SegmentFile};
    use protocol::storage::protocol::ReadReqOptions;

    #[tokio::test]
    async fn read_by_offset_test() {
        let (segment_iden, _, fold, rocksdb_engine_handler) = test_base_write_data(30).await;
        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold)
                .await
                .unwrap();

        let max_record = 2;
        let max_size = 1024 * 1024 * 1024;
        let resp = segment_read_by_offset(
            &rocksdb_engine_handler,
            &segment_file,
            &segment_iden.shard_name,
            5,
            max_size,
            max_record,
        )
        .await
        .unwrap();

        assert_eq!(resp.len(), 2);

        let mut i = 5;
        for row in resp {
            assert_eq!(row.record.metadata.key.unwrap(), format!("key-{i}"));
            i += 1;
        }

        let max_record = 5;
        let resp = segment_read_by_offset(
            &rocksdb_engine_handler,
            &segment_file,
            &segment_iden.shard_name,
            10,
            max_size,
            max_record,
        )
        .await
        .unwrap();
        assert_eq!(resp.len(), 5);

        let mut i: i32 = 10;
        for row in resp {
            assert_eq!(row.record.metadata.key.unwrap(), format!("key-{i}"));
            i += 1;
        }
    }

    #[tokio::test]
    async fn read_by_key_test() {
        let (segment_iden, _, fold, rocksdb_engine_handler) = test_base_write_data(30).await;

        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold)
                .await
                .unwrap();

        let key = "key-5".to_string();
        let resp = segment_read_by_key(
            &rocksdb_engine_handler,
            &segment_file,
            &segment_iden.shard_name,
            &key,
        )
        .await
        .unwrap();

        assert_eq!(resp.len(), 1);
        let meata = resp.first().unwrap().record.metadata.clone();
        assert_eq!(meata.key.unwrap(), key);
    }

    #[tokio::test]
    async fn read_by_tag_test() {
        let (segment_iden, _, fold, rocksdb_engine_handler) = test_base_write_data(30).await;

        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold)
                .await
                .unwrap();

        let read_options = ReadReqOptions {
            max_record: 10,
            max_size: 1024 * 1024 * 1024,
        };

        let tag = "tag-5".to_string();
        let res = segment_read_by_tag(
            &rocksdb_engine_handler,
            &segment_file,
            &segment_iden.shard_name,
            &tag,
            None,
            read_options.max_record,
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
