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
use crate::{
    core::{cache::StorageCacheManager, error::StorageEngineError},
    filesegment::{
        file::{open_segment_write, ReadData},
        index::read::{get_index_data_by_key, get_index_data_by_offset, get_index_data_by_tag},
    },
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::{collections::HashMap, sync::Arc};

/// handle read requests by offset
///
/// Use index (if there's any) to find the last nearest start byte position given the offset
pub async fn segment_read_by_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    offset: u64,
    max_size: u64,
    max_record: u64,
) -> Result<Vec<ReadData>, StorageEngineError> {
    let start_position = if let Some(position) =
        get_index_data_by_offset(rocksdb_engine_handler, segment_iden, offset)?
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
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    key: &str,
) -> Result<Vec<ReadData>, StorageEngineError> {
    let index_data = get_index_data_by_key(rocksdb_engine_handler, shard_name, key.to_string())?;

    if let Some(index) = index_data {
        let segment_iden = SegmentIdentity::new(shard_name, index.segment);
        let segment_file = open_segment_write(cache_manager, &segment_iden).await?;
        return segment_file.read_by_positions(vec![index.position]).await;
    }
    Ok(Vec::new())
}

pub async fn segment_read_by_tag(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
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

    let mut segment_positions = HashMap::new();

    for index_data in index_data_list {
        segment_positions
            .entry(index_data.segment)
            .or_insert_with(Vec::new)
            .push(index_data.position);
    }

    let mut all_results = Vec::new();

    for (segment_no, positions) in segment_positions {
        let segment_iden = SegmentIdentity::new(shard_name, segment_no);
        let segment_file = open_segment_write(cache_manager, &segment_iden).await?;
        let data_list = segment_file.read_by_positions(positions).await?;
        all_results.extend(data_list);
    }

    Ok(all_results)
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use super::{segment_read_by_key, segment_read_by_offset, segment_read_by_tag};
    use crate::{
        commitlog::offset::CommitLogOffset,
        core::{cache::StorageCacheManager, test_tool::test_init_segment},
        filesegment::{
            file::SegmentFile,
            write::{WriteChannelDataRecord, WriteManager},
            SegmentIdentity,
        },
    };
    use bytes::Bytes;
    use common_config::storage::StorageType;
    use grpc_clients::pool::ClientPool;
    use protocol::storage::protocol::ReadReqOptions;
    use rocksdb_engine::rocksdb::RocksDBEngine;
    use tokio::{sync::broadcast, time::sleep};

    #[allow(dead_code)]
    pub async fn test_base_write_data(
        engine_storage_type: StorageType,
        len: u64,
    ) -> (
        SegmentIdentity,
        Arc<StorageCacheManager>,
        String,
        Arc<RocksDBEngine>,
    ) {
        let (segment_iden, cache_manager, fold, rocksdb_engine_handler) =
            test_init_segment(engine_storage_type).await;

        let client_poll = Arc::new(ClientPool::new(100));

        let write_manager = WriteManager::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
            client_poll.clone(),
            3,
        );

        let (stop_send, _) = broadcast::channel(2);
        write_manager.start(stop_send.clone());

        sleep(Duration::from_millis(100)).await;

        let mut data_list = Vec::new();
        for i in 0..len {
            data_list.push(WriteChannelDataRecord {
                pkid: i,
                header: None,
                key: Some(format!("key-{}", i)),
                tags: Some(vec![format!("tag-{}", i)]),
                value: Bytes::from(format!("data-{i}")),
            });
        }

        let _res = write_manager.write(&segment_iden, data_list).await.unwrap();
        stop_send.send(true).ok();
        sleep(Duration::from_millis(100)).await;

        (segment_iden, cache_manager, fold, rocksdb_engine_handler)
    }

    #[tokio::test]
    async fn read_by_offset_test() {
        let (segment_iden, cache_manager, fold, rocksdb_engine_handler) =
            test_base_write_data(StorageType::EngineSegment, 30).await;
        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold)
                .await
                .unwrap();
        let commit_offset =
            CommitLogOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
        commit_offset
            .save_earliest_offset(&segment_iden.shard_name, 0)
            .unwrap();
        commit_offset
            .save_latest_offset(&segment_iden.shard_name, 0)
            .unwrap();

        let max_record = 2;
        let max_size = 1024 * 1024 * 1024;
        let resp = segment_read_by_offset(
            &rocksdb_engine_handler,
            &segment_file,
            &segment_iden,
            5,
            max_size,
            max_record,
        )
        .await
        .unwrap();

        assert_eq!(resp.len(), 2);

        let mut i = 5;
        for row in resp {
            assert_eq!(row.record.metadata.key.unwrap(), format!("key-{}", i));
            i += 1;
        }

        let max_record = 5;
        let resp = segment_read_by_offset(
            &rocksdb_engine_handler,
            &segment_file,
            &segment_iden,
            10,
            max_size,
            max_record,
        )
        .await
        .unwrap();
        assert_eq!(resp.len(), 5);

        let mut i: i32 = 10;
        for row in resp {
            assert_eq!(row.record.metadata.key.unwrap(), format!("key-{}", i));
            i += 1;
        }
    }

    #[tokio::test]
    async fn read_by_key_test() {
        let (segment_iden, cache_manager, _, rocksdb_engine_handler) =
            test_base_write_data(StorageType::EngineSegment, 30).await;

        let key = "key-5".to_string();
        let resp = segment_read_by_key(
            &cache_manager,
            &rocksdb_engine_handler,
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
        let (segment_iden, cache_manager, _, rocksdb_engine_handler) =
            test_base_write_data(StorageType::EngineSegment, 30).await;

        let read_options = ReadReqOptions {
            max_record: 10,
            max_size: 1024 * 1024 * 1024,
        };

        let tag = "tag-5".to_string();
        let res = segment_read_by_tag(
            &cache_manager,
            &rocksdb_engine_handler,
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
