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

use std::sync::Arc;

use protocol::journal::journal_engine::{
    ReadReqBody, ReadReqFilter, ReadReqOptions, ReadRespMessage, ReadRespSegmentMessage, ReadType,
};
use rocksdb_engine::rocksdb::RocksDBEngine;

use super::file::{ReadData, SegmentFile};
use super::SegmentIdentity;
use crate::core1::cache::CacheManager;
use crate::core1::error::JournalServerError;
use crate::index::offset::OffsetIndexManager;
use crate::index::tag::TagIndexManager;

/// handle all read requests from Journal Client
///
/// Redirect read requests to the corresponding handler according to the read type
pub async fn read_data_req(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req_body: &ReadReqBody,
    node_id: u64,
) -> Result<Vec<ReadRespSegmentMessage>, JournalServerError> {
    let mut results = Vec::new();
    for raw in req_body.messages.iter() {
        let mut shard_message = ReadRespSegmentMessage {
            shard_name: raw.shard_name.to_string(),
            segment: raw.segment,
            ..Default::default()
        };

        let segment_iden = SegmentIdentity {
            shard_name: raw.shard_name.to_string(),
            segment_seq: raw.segment,
        };

        let segment = if let Some(segment) = cache_manager.get_segment(&segment_iden) {
            segment
        } else {
            return Err(JournalServerError::SegmentNotExist(segment_iden.name()));
        };

        let fold = if let Some(fold) = segment.get_fold(node_id) {
            fold
        } else {
            return Err(JournalServerError::SegmentDataDirectoryNotFound(
                segment_iden.name(),
                node_id,
            ));
        };

        let segment_file = SegmentFile::new(
            segment_iden.shard_name.clone(),
            segment_iden.segment_seq,
            fold,
        );

        let filter = if let Some(filter) = raw.filter.clone() {
            filter
        } else {
            ReadReqFilter {
                offset: 0,
                ..Default::default()
            }
        };

        let read_options = if let Some(option) = raw.options {
            option
        } else {
            ReadReqOptions {
                max_size: 1024 * 1024,
                max_record: 100,
            }
        };

        let read_data_list = match raw.ready_type() {
            ReadType::Offset => {
                read_by_offset(
                    rocksdb_engine_handler,
                    &segment_file,
                    &segment_iden,
                    &filter,
                    &read_options,
                )
                .await?
            }

            ReadType::Key => {
                read_by_key(
                    rocksdb_engine_handler,
                    &segment_file,
                    &segment_iden,
                    &filter,
                    &read_options,
                )
                .await?
            }

            ReadType::Tag => {
                read_by_tag(
                    rocksdb_engine_handler,
                    &segment_file,
                    &segment_iden,
                    &filter,
                    &read_options,
                )
                .await?
            }
        };

        let mut record_message = Vec::new();
        for read_data in read_data_list {
            let record = read_data.record;
            record_message.push(ReadRespMessage {
                offset: record.offset as u64,
                key: record.key,
                value: record.content,
                tags: record.tags,
                timestamp: record.create_time,
            });
        }
        shard_message.messages = record_message;

        results.push(shard_message);
    }
    Ok(results)
}

/// handle read requests by offset
///
/// Use index (if there's any) to find the last nearest start byte position given the offset
async fn read_by_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    filter: &ReadReqFilter,
    read_options: &ReadReqOptions,
) -> Result<Vec<ReadData>, JournalServerError> {
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
async fn read_by_key(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    filter: &ReadReqFilter,
    read_options: &ReadReqOptions,
) -> Result<Vec<ReadData>, JournalServerError> {
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
async fn read_by_tag(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    filter: &ReadReqFilter,
    read_options: &ReadReqOptions,
) -> Result<Vec<ReadData>, JournalServerError> {
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
    use std::time::Duration;

    use common_config::broker::broker_config;
    use protocol::journal::journal_engine::{
        ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType,
    };
    use tokio::time::sleep;

    use super::{read_by_key, read_by_offset, read_by_tag, read_data_req};
    use crate::core1::test::test_base_write_data;
    use crate::index::build::try_trigger_build_index;
    use crate::segment::file::SegmentFile;

    #[tokio::test]
    async fn read_by_offset_test() {
        let (segment_iden, _, _, fold, rocksdb_engine_handler) = test_base_write_data(30).await;

        let segment_file = SegmentFile::new(
            segment_iden.shard_name.clone(),
            segment_iden.segment_seq,
            fold,
        );

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
            assert_eq!(row.record.key, format!("key-{i}"));
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
            println!("{row:?}");
            assert_eq!(row.record.key, format!("key-{i}"));
            i += 1;
        }
    }

    #[tokio::test]
    async fn read_by_key_test() {
        let (segment_iden, cache_manager, segment_file_manager, fold, rocksdb_engine_handler) =
            test_base_write_data(30).await;
        let res = try_trigger_build_index(
            &cache_manager,
            &segment_file_manager,
            &rocksdb_engine_handler,
            &segment_iden,
        )
        .await;
        assert!(res.is_ok());

        sleep(Duration::from_secs(10)).await;

        let segment_file = SegmentFile::new(
            segment_iden.shard_name.clone(),
            segment_iden.segment_seq,
            fold,
        );

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
        assert_eq!(resp.first().unwrap().record.key, key);
    }

    #[tokio::test]
    async fn read_by_tag_test() {
        let (segment_iden, cache_manager, segment_file_manager, fold, rocksdb_engine_handler) =
            test_base_write_data(30).await;

        let res = try_trigger_build_index(
            &cache_manager,
            &segment_file_manager,
            &rocksdb_engine_handler,
            &segment_iden,
        )
        .await;
        assert!(res.is_ok());

        sleep(Duration::from_secs(10)).await;

        let segment_file = SegmentFile::new(
            segment_iden.shard_name.clone(),
            segment_iden.segment_seq,
            fold,
        );

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
        assert!(resp.first().unwrap().record.tags.contains(&tag));
    }

    #[tokio::test]
    async fn read_data_req_test() {
        let (segment_iden, cache_manager, segment_file_manager, _, rocksdb_engine_handler) =
            test_base_write_data(30).await;

        let res = try_trigger_build_index(
            &cache_manager,
            &segment_file_manager,
            &rocksdb_engine_handler,
            &segment_iden,
        )
        .await;
        assert!(res.is_ok());

        sleep(Duration::from_secs(10)).await;

        // offset
        let req_body = ReadReqBody {
            messages: vec![ReadReqMessage {
                shard_name: segment_iden.shard_name.clone(),
                segment: segment_iden.segment_seq,
                ready_type: ReadType::Offset.into(),
                filter: Some(ReadReqFilter {
                    offset: 5,
                    ..Default::default()
                }),
                options: Some(ReadReqOptions {
                    max_size: 1024 * 1024 * 1024,
                    max_record: 2,
                }),
            }],
        };
        let conf = broker_config();
        let res = read_data_req(
            &cache_manager,
            &rocksdb_engine_handler,
            &req_body,
            conf.broker_id,
        )
        .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert_eq!(resp.len(), 1);
        let resp_shard = resp.first().unwrap();
        assert_eq!(resp_shard.messages.len(), 2);

        let mut i = 5;
        for row in resp_shard.messages.iter() {
            assert_eq!(row.offset, i);
            i += 1;
        }

        // key
        let key = format!("key-{}", 1);
        let req_body = ReadReqBody {
            messages: vec![ReadReqMessage {
                shard_name: segment_iden.shard_name.clone(),
                segment: segment_iden.segment_seq,
                ready_type: ReadType::Key.into(),
                filter: Some(ReadReqFilter {
                    offset: 0,
                    key: key.clone(),
                    ..Default::default()
                }),
                options: Some(ReadReqOptions {
                    max_size: 1024 * 1024 * 1024,
                    max_record: 2,
                }),
            }],
        };
        let conf = broker_config();
        let res = read_data_req(
            &cache_manager,
            &rocksdb_engine_handler,
            &req_body,
            conf.broker_id,
        )
        .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert_eq!(resp.len(), 1);
        let resp_shard = resp.first().unwrap();
        assert_eq!(resp_shard.messages.len(), 1);
        let data = resp_shard.messages.first().unwrap();
        assert_eq!(data.key, key);

        // tag
        let tag = format!("tag-{}", 1);
        let req_body = ReadReqBody {
            messages: vec![ReadReqMessage {
                shard_name: segment_iden.shard_name.clone(),
                segment: segment_iden.segment_seq,
                ready_type: ReadType::Tag.into(),
                filter: Some(ReadReqFilter {
                    offset: 0,
                    tag: tag.clone(),
                    ..Default::default()
                }),
                options: Some(ReadReqOptions {
                    max_size: 1024 * 1024 * 1024,
                    max_record: 2,
                }),
            }],
        };
        let conf = broker_config();
        let res = read_data_req(
            &cache_manager,
            &rocksdb_engine_handler,
            &req_body,
            conf.broker_id,
        )
        .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert_eq!(resp.len(), 1);
        let resp_shard = resp.first().unwrap();
        assert_eq!(resp_shard.messages.len(), 1);
        let data = resp_shard.messages.first().unwrap();
        assert!(data.tags.contains(&tag));
    }
}
