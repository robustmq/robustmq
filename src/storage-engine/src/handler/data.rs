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
use crate::segment::file::SegmentFile;
use crate::segment::read::{read_by_key, read_by_offset, read_by_tag};
use crate::segment::write::{WriteChannelDataRecord, WriteManager};
use crate::segment::SegmentIdentity;
use common_config::broker::broker_config;
use protocol::storage::protocol::{
    ReadReqBody, ReadRespMessage, ReadRespSegmentMessage, ReadType, WriteReqMessages,
    WriteRespMessage, WriteRespMessageStatus,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

fn params_validator(
    cache_manager: &Arc<StorageCacheManager>,
    segment_identity: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    if !cache_manager
        .shards
        .contains_key(&segment_identity.shard_name)
    {
        return Err(StorageEngineError::ShardNotExist(
            segment_identity.shard_name.to_string(),
        ));
    }

    let segment = if let Some(segment) = cache_manager.get_segment(segment_identity) {
        segment
    } else {
        return Err(StorageEngineError::SegmentNotExist(segment_identity.name()));
    };

    if !segment.allow_read() {
        return Err(StorageEngineError::SegmentStatusError(
            segment_identity.name(),
            segment.status.to_string(),
        ));
    }

    let conf = broker_config();
    if segment.leader != conf.broker_id {
        return Err(StorageEngineError::NotLeader(segment_identity.name()));
    }

    if cache_manager.get_segment_meta(segment_identity).is_none() {
        return Err(StorageEngineError::SegmentFileMetaNotExists(
            segment_identity.name(),
        ));
    }

    Ok(())
}

/// the entry point for handling write requests
pub async fn write_data_req(
    cache_manager: &Arc<StorageCacheManager>,
    write_manager: &Arc<WriteManager>,
    segment_iden: &SegmentIdentity,
    messages: &[WriteReqMessages],
) -> Result<Vec<WriteRespMessage>, StorageEngineError> {
    if messages.is_empty() {
        return Ok(Vec::new());
    }

    params_validator(cache_manager, segment_iden)?;

    let mut results = Vec::new();

    let mut record_list = Vec::new();
    for message in messages {
        // todo data validator
        let record = WriteChannelDataRecord {
            pkid: message.pkid,
            header: None,
            key: Some(message.key.clone()),
            tags: Some(message.tags.clone()),
            value: message.value.clone().into(),
        };
        record_list.push(record);
    }

    let response = write_manager.write(segment_iden, record_list).await?;

    let resp_message = WriteRespMessage {
        shard_name: segment_iden.shard_name.clone(),
        segment: segment_iden.segment,
        messages: response
            .offsets
            .iter()
            .map(|(pkid, offset)| WriteRespMessageStatus {
                pkid: *pkid,
                offset: *offset,
                ..Default::default()
            })
            .collect(),
    };

    results.push(resp_message);

    Ok(results)
}

/// handle all read requests from Journal Client
///
/// Redirect read requests to the corresponding handler according to the read type
pub async fn read_data_req(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req_body: &ReadReqBody,
    node_id: u64,
) -> Result<Vec<ReadRespSegmentMessage>, StorageEngineError> {
    let mut results = Vec::new();
    for raw in req_body.messages.iter() {
        let mut shard_message = ReadRespSegmentMessage {
            shard_name: raw.shard_name.to_string(),
            segment: raw.segment,
            ..Default::default()
        };

        let segment_iden = SegmentIdentity {
            shard_name: raw.shard_name.to_string(),
            segment: raw.segment,
        };

        let segment = if let Some(segment) = cache_manager.get_segment(&segment_iden) {
            segment
        } else {
            return Err(StorageEngineError::SegmentNotExist(segment_iden.name()));
        };

        let fold = if let Some(fold) = segment.get_fold(node_id) {
            fold
        } else {
            return Err(StorageEngineError::SegmentDataDirectoryNotFound(
                segment_iden.name(),
                node_id,
            ));
        };

        let segment_file =
            SegmentFile::new(segment_iden.shard_name.clone(), segment_iden.segment, fold).await?;

        let filter = raw.filter.clone();

        let read_options = raw.options.clone();

        let read_data_list = match raw.read_type {
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
                offset: record.metadata.offset,
                key: record.metadata.key,
                value: record.data.to_vec(),
                tags: record.metadata.tags,
                timestamp: record.metadata.create_t,
            });
        }
        shard_message.messages = record_message;

        results.push(shard_message);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use common_config::broker::broker_config;
    use protocol::storage::protocol::{
        ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType,
    };
    use tokio::time::sleep;

    use crate::{core::test::test_base_write_data, handler::data::read_data_req};

    #[tokio::test]
    #[ignore]
    async fn read_data_req_test() {
        let (segment_iden, cache_manager, _, rocksdb_engine_handler) =
            test_base_write_data(30).await;

        sleep(Duration::from_secs(10)).await;

        // offset
        let req_body = ReadReqBody {
            messages: vec![ReadReqMessage {
                shard_name: segment_iden.shard_name.clone(),
                segment: segment_iden.segment,
                read_type: ReadType::Offset,
                filter: ReadReqFilter {
                    offset: 5,
                    ..Default::default()
                },
                options: ReadReqOptions {
                    max_size: 1024 * 1024 * 1024,
                    max_record: 2,
                },
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
                segment: segment_iden.segment,
                read_type: ReadType::Key,
                filter: ReadReqFilter {
                    offset: 0,
                    key: key.clone(),
                    ..Default::default()
                },
                options: ReadReqOptions {
                    max_size: 1024 * 1024 * 1024,
                    max_record: 2,
                },
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
        let data = resp_shard.messages.first().unwrap().clone();
        assert_eq!(data.key.unwrap(), key);

        // tag
        let tag = format!("tag-{}", 1);
        let req_body = ReadReqBody {
            messages: vec![ReadReqMessage {
                shard_name: segment_iden.shard_name.clone(),
                segment: segment_iden.segment,
                read_type: ReadType::Tag,
                filter: ReadReqFilter {
                    offset: 0,
                    tag: tag.clone(),
                    ..Default::default()
                },
                options: ReadReqOptions {
                    max_size: 1024 * 1024 * 1024,
                    max_record: 2,
                },
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
        let data = resp_shard.messages.first().unwrap().clone();
        assert!(data.tags.unwrap().contains(&tag));
    }
}
