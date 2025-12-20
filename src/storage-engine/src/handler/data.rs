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
use crate::core::error::{get_journal_server_code, StorageEngineError};
use crate::core::segment_meta::{update_meta_end_timestamp, update_meta_start_timestamp};
use crate::core::segment_status::sealup_segment;
use crate::core::shard::try_auto_create_shard;
use crate::segment::file::SegmentFile;
use crate::segment::manager::SegmentFileManager;
use crate::segment::read::{read_by_key, read_by_offset, read_by_tag};
use crate::segment::write::{get_write, write_data};
use crate::segment::SegmentIdentity;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use protocol::storage::storage_engine_engine::{
    ReadReq, ReadReqBody, ReadReqFilter, ReadReqOptions, ReadRespMessage, ReadRespSegmentMessage,
    ReadType, WriteReq, WriteReqBody, WriteRespMessage, WriteRespMessageStatus,
};
use protocol::storage::storage_engine_record::StorageEngineRecord;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

#[derive(Clone)]
pub struct DataHandler {
    cache_manager: Arc<StorageCacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
}

impl DataHandler {
    pub fn new(
        cache_manager: Arc<StorageCacheManager>,
        segment_file_manager: Arc<SegmentFileManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_pool: Arc<ClientPool>,
    ) -> DataHandler {
        DataHandler {
            cache_manager,
            segment_file_manager,
            rocksdb_engine_handler,
            client_pool,
        }
    }

    pub async fn write(
        &self,
        request: WriteReq,
    ) -> Result<Vec<WriteRespMessage>, StorageEngineError> {
        if request.body.is_none() {
            return Err(StorageEngineError::RequestBodyNotEmpty("write".to_string()));
        }

        let req_body = request.body.unwrap();
        for message in req_body.data.iter() {
            try_auto_create_shard(&self.cache_manager, &self.client_pool, &message.shard_name)
                .await?;

            let segment_identity = SegmentIdentity {
                shard_name: message.shard_name.to_string(),
                segment_seq: message.segment,
            };
            self.validator(&segment_identity)?;
        }

        let results = write_data_req(
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            &self.segment_file_manager,
            &self.client_pool,
            &req_body,
        )
        .await?;
        Ok(results)
    }

    pub async fn read(
        &self,
        request: ReadReq,
    ) -> Result<Vec<ReadRespSegmentMessage>, StorageEngineError> {
        if request.body.is_none() {
            return Err(StorageEngineError::RequestBodyNotEmpty("write".to_string()));
        }

        let req_body = request.body.unwrap();
        for row in req_body.messages.clone() {
            try_auto_create_shard(&self.cache_manager, &self.client_pool, &row.shard_name).await?;

            let segment_identity = SegmentIdentity {
                shard_name: row.shard_name.to_string(),
                segment_seq: row.segment,
            };
            self.validator(&segment_identity)?;
        }

        let conf = broker_config();
        let results = read_data_req(
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            &req_body,
            conf.broker_id,
        )
        .await?;
        Ok(results)
    }

    fn validator(&self, segment_identity: &SegmentIdentity) -> Result<(), StorageEngineError> {
        if !self
            .cache_manager
            .shards
            .contains_key(&segment_identity.shard_name)
        {
            return Err(StorageEngineError::ShardNotExist(
                segment_identity.shard_name.to_string(),
            ));
        }

        let segment = if let Some(segment) = self.cache_manager.get_segment(segment_identity) {
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

        if self
            .cache_manager
            .get_segment_meta(segment_identity)
            .is_none()
        {
            return Err(StorageEngineError::SegmentFileMetaNotExists(
                segment_identity.name(),
            ));
        }

        Ok(())
    }
}

/// the entry point for handling write requests
pub async fn write_data_req(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    client_pool: &Arc<ClientPool>,
    req_body: &WriteReqBody,
) -> Result<Vec<WriteRespMessage>, StorageEngineError> {
    let mut results = Vec::new();
    for shard_data in req_body.data.clone() {
        let mut resp_message = WriteRespMessage {
            shard_name: shard_data.shard_name.clone(),
            segment: shard_data.segment,
            ..Default::default()
        };

        let segment_iden = SegmentIdentity::new(&shard_data.shard_name, shard_data.segment);

        let mut record_list = Vec::new();
        for message in shard_data.messages.iter() {
            // todo data validator
            let record = StorageEngineRecord {
                content: message.value.clone(),
                create_time: now_second(),
                key: message.key.clone(),
                shard_name: shard_data.shard_name.clone(),
                segment: shard_data.segment,
                tags: message.tags.clone(),
                pkid: message.pkid,
                producer_id: "".to_string(),
                offset: -1,
            };
            record_list.push(record);
        }

        let resp = match write_data(
            cache_manager,
            rocksdb_engine_handler,
            segment_file_manager,
            &segment_iden,
            record_list.clone(),
        )
        .await
        {
            Ok(resp) => resp,
            Err(e) => {
                // if this write filled up the segment, we need to seal up the segment and update end timestamp
                if get_journal_server_code(&e) == *"SegmentOffsetAtTheEnd" {
                    sealup_segment(cache_manager, client_pool, &segment_iden).await?;
                    update_meta_end_timestamp(client_pool, &segment_iden, segment_file_manager)
                        .await?;
                    let write = get_write(
                        cache_manager,
                        rocksdb_engine_handler,
                        segment_file_manager,
                        &segment_iden,
                    )
                    .await?;

                    // Stop the segment writer thread while waiting for messages to be written to the channel to clear
                    loop {
                        if write.data_sender.capacity() == write.data_sender.max_capacity() {
                            write.stop_sender.send(true)?;
                            break;
                        }
                        sleep(Duration::from_millis(10)).await;
                    }
                }

                return Err(e);
            }
        };

        if let Some(e) = resp.error {
            return Err(e);
        }

        let mut resp_message_status = Vec::new();
        for (pkid, offset) in resp.offsets {
            let status = WriteRespMessageStatus {
                pkid,
                offset,
                ..Default::default()
            };
            resp_message_status.push(status);
            let segment_file_meta = segment_file_manager
                .get_segment_file(&segment_iden)
                .unwrap();

            // TODO: When it will happen?
            if segment_file_meta.start_offset as u64 == offset {
                let mut record = None;
                for rc in record_list.iter() {
                    if rc.pkid == pkid {
                        record = Some(rc.clone());
                    }
                }
                if let Some(rc) = record {
                    let start_timestamp = rc.create_time;
                    segment_file_manager.update_start_offset(&segment_iden, offset as i64)?;
                    segment_file_manager.update_start_timestamp(&segment_iden, start_timestamp)?;
                    update_meta_start_timestamp(client_pool, &segment_iden, start_timestamp)
                        .await?;
                } else {
                    warn!("");
                }
            }
        }
        resp_message.messages = resp_message_status;
        results.push(resp_message);
    }
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
            segment_seq: raw.segment,
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use common_config::broker::broker_config;
    use protocol::storage::storage_engine_engine::{
        ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType,
    };
    use tokio::time::sleep;

    use crate::{
        core::test::test_base_write_data, handler::data::read_data_req,
        segment::index::build::try_trigger_build_index,
    };

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
