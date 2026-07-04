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
    core::{
        cache::StorageCacheManager, error::StorageEngineError, message_ttl::is_record_expired,
        offset::ShardOffset,
    },
    filesegment::{
        file::{open_segment_write, ReadData},
        index::read::{get_index_data_by_key, get_index_data_by_offset, get_index_data_by_tag},
    },
};
use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::{collections::HashMap, sync::Arc};

pub async fn segment_read_by_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &mut SegmentFile,
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
    let res: Vec<ReadData> = res
        .into_iter()
        .filter(|r| !is_record_expired(&r.record.metadata))
        .collect();
    Ok(res)
}

pub async fn segment_read_by_key(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    key: &[u8],
) -> Result<Vec<ReadData>, StorageEngineError> {
    let index_data = get_index_data_by_key(rocksdb_engine_handler, shard_name, key)?;

    if let Some(index) = index_data {
        let segment_iden = SegmentIdentity::new(shard_name, index.segment);
        let mut segment_file = open_segment_write(cache_manager, &segment_iden).await?;
        let res = segment_file.read_by_positions(vec![index.position]).await?;
        let res: Vec<ReadData> = res
            .into_iter()
            .filter(|r| !is_record_expired(&r.record.metadata))
            .collect();
        return Ok(res);
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
        let mut segment_file = open_segment_write(cache_manager, &segment_iden).await?;
        let data_list = segment_file.read_by_positions(positions).await?;
        all_results.extend(
            data_list
                .into_iter()
                .filter(|r| !is_record_expired(&r.record.metadata)),
        );
    }

    Ok(all_results)
}

pub fn get_segment_offset_by_timestamp(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    timestamp: u64,
    strategy: AdapterOffsetStrategy,
) -> Result<u64, StorageEngineError> {
    use crate::filesegment::index::read::{
        get_in_segment_by_timestamp, get_index_data_by_timestamp,
    };

    if let Some(segment) = get_in_segment_by_timestamp(cache_manager, shard_name, timestamp as i64)?
    {
        let segment_iden = SegmentIdentity::new(shard_name, segment);
        if let Some(index_data) =
            get_index_data_by_timestamp(rocksdb_engine_handler, &segment_iden, timestamp)?
        {
            return Ok(index_data.offset);
        }
        return Err(StorageEngineError::CommonErrorStr(format!(
            "No index data found for timestamp {} in segment {}",
            timestamp, segment
        )));
    }
    let shard_offset = ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
    match strategy {
        AdapterOffsetStrategy::Earliest => shard_offset.get_earliest_offset(shard_name),
        AdapterOffsetStrategy::Latest => shard_offset.get_latest_offset(shard_name),
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use super::{segment_read_by_key, segment_read_by_offset, segment_read_by_tag};
    use crate::{
        core::offset::ShardOffset,
        core::{cache::StorageCacheManager, test_tool::test_init_segment},
        filesegment::{
            file::SegmentFile,
            write_manager::{WriteChannelDataRecord, WriteManager},
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
                key: Some(format!("key-{}", i).into()),
                tags: Some(vec![format!("tag-{}", i)]),
                value: Bytes::from(format!("data-{i}")),
                protocol_data: None,
                expire_at: 0,
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
        let mut segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold)
                .await
                .unwrap();
        let commit_offset = ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
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
            &mut segment_file,
            &segment_iden,
            5,
            max_size,
            max_record,
        )
        .await
        .unwrap();

        assert_eq!(resp.len(), 2);

        for (i, row) in (5..).zip(resp) {
            assert_eq!(
                row.record.metadata.key.unwrap(),
                bytes::Bytes::from(format!("key-{}", i))
            );
        }

        let max_record = 5;
        let resp = segment_read_by_offset(
            &rocksdb_engine_handler,
            &mut segment_file,
            &segment_iden,
            10,
            max_size,
            max_record,
        )
        .await
        .unwrap();
        assert_eq!(resp.len(), 5);

        for (i, row) in (10_i32..).zip(resp) {
            assert_eq!(
                row.record.metadata.key.unwrap(),
                bytes::Bytes::from(format!("key-{}", i))
            );
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
            key.as_bytes(),
        )
        .await
        .unwrap();

        assert_eq!(resp.len(), 1);
        let meata = resp.first().unwrap().record.metadata.clone();
        assert_eq!(meata.key.unwrap(), key.as_bytes());
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
