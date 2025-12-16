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

use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use protocol::journal::journal_engine::{
    FetchOffsetReq, FetchOffsetRespBody, FetchOffsetShardMeta, JournalEngineError, ReadReq,
    ReadRespSegmentMessage, WriteReq, WriteRespMessage,
};
use rocksdb_engine::rocksdb::RocksDBEngine;

use crate::core::cache::CacheManager;
use crate::core::error::{get_journal_server_code, JournalServerError};
use crate::core::shard::try_auto_create_shard;
use crate::index::time::TimestampIndexManager;
use crate::segment::manager::SegmentFileManager;
use crate::segment::read::read_data_req;
use crate::segment::write::write_data_req;
use crate::segment::SegmentIdentity;

#[derive(Clone)]
pub struct DataHandler {
    cache_manager: Arc<CacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
}

impl DataHandler {
    pub fn new(
        cache_manager: Arc<CacheManager>,
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
    ) -> Result<Vec<WriteRespMessage>, JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty("write".to_string()));
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
    ) -> Result<Vec<ReadRespSegmentMessage>, JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty("write".to_string()));
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

    pub async fn fetch_offset(
        &self,
        request: FetchOffsetReq,
    ) -> Result<FetchOffsetRespBody, JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty(
                "fetch_offset".to_string(),
            ));
        }

        let req_body = request.body.unwrap();
        let mut meta_list: Vec<FetchOffsetShardMeta> = Vec::new();

        for shard in req_body.shards {
            try_auto_create_shard(&self.cache_manager, &self.client_pool, &shard.shard_name)
                .await?;
            let segment_iden = SegmentIdentity {
                shard_name: shard.shard_name.clone(),
                segment_seq: shard.segment_no,
            };

            let file_metadata = self.cache_manager.get_segment_meta(&segment_iden);
            let segment_meta = if let Some(meta) = file_metadata {
                meta
            } else {
                let e = &JournalServerError::SegmentMetaNotExists(segment_iden.name());
                meta_list.push(FetchOffsetShardMeta {
                    error: Some(JournalEngineError {
                        code: get_journal_server_code(e),
                        error: e.to_string(),
                    }),
                    ..Default::default()
                });
                continue;
            };

            let active_segment =
                if let Some(segment) = self.cache_manager.get_active_segment(&shard.shard_name) {
                    segment.segment_seq
                } else {
                    let e = &JournalServerError::NotActiveSegment(shard.shard_name);
                    meta_list.push(FetchOffsetShardMeta {
                        error: Some(JournalEngineError {
                            code: get_journal_server_code(e),
                            error: e.to_string(),
                        }),
                        ..Default::default()
                    });
                    continue;
                };

            let is_active_segment = active_segment == shard.segment_no;

            if shard.timestamp < segment_meta.start_timestamp as u64 {
                let e = &JournalServerError::TimestampBelongToPreviousSegment(
                    shard.timestamp,
                    segment_meta.start_offset,
                    shard.shard_name,
                );
                meta_list.push(FetchOffsetShardMeta {
                    error: Some(JournalEngineError {
                        code: get_journal_server_code(e),
                        error: e.to_string(),
                    }),
                    ..Default::default()
                });
                continue;
            };

            if shard.timestamp > segment_meta.end_timestamp as u64 {
                if is_active_segment {
                    let meta = FetchOffsetShardMeta {
                        shard_name: shard.shard_name,
                        segment_no: shard.segment_no,
                        offset: segment_meta.end_offset,
                        ..Default::default()
                    };

                    meta_list.push(meta);
                    continue;
                } else {
                    let e = &JournalServerError::TimestampBelongToNextSegment(
                        shard.timestamp,
                        segment_meta.start_offset,
                        shard.shard_name,
                    );
                    meta_list.push(FetchOffsetShardMeta {
                        error: Some(JournalEngineError {
                            code: get_journal_server_code(e),
                            error: e.to_string(),
                        }),
                        ..Default::default()
                    });
                    continue;
                }
            }

            let timestamp_index = TimestampIndexManager::new(self.rocksdb_engine_handler.clone());
            let offset = if let Some(index_data) = timestamp_index
                .get_last_nearest_position_by_timestamp(&segment_iden, shard.timestamp)
                .await?
            {
                index_data.offset
            } else {
                let e = &JournalServerError::NotAvailableOffsetByTimestamp(
                    shard.timestamp,
                    shard.shard_name,
                );
                meta_list.push(FetchOffsetShardMeta {
                    error: Some(JournalEngineError {
                        code: get_journal_server_code(e),
                        error: e.to_string(),
                    }),
                    ..Default::default()
                });
                continue;
            };

            let meta = FetchOffsetShardMeta {
                shard_name: shard.shard_name,
                segment_no: shard.segment_no,
                offset: offset as i64,
                ..Default::default()
            };

            meta_list.push(meta);
        }

        Ok(FetchOffsetRespBody {
            shard_offsets: meta_list,
        })
    }

    fn validator(&self, segment_identity: &SegmentIdentity) -> Result<(), JournalServerError> {
        if self
            .cache_manager
            .get_shard(&segment_identity.shard_name)
            .is_none()
        {
            return Err(JournalServerError::ShardNotExist(
                segment_identity.shard_name.to_string(),
            ));
        }

        let segment = if let Some(segment) = self.cache_manager.get_segment(segment_identity) {
            segment
        } else {
            return Err(JournalServerError::SegmentNotExist(segment_identity.name()));
        };

        if !segment.allow_read() {
            return Err(JournalServerError::SegmentStatusError(
                segment_identity.name(),
                segment.status.to_string(),
            ));
        }

        let conf = broker_config();
        if segment.leader != conf.broker_id {
            return Err(JournalServerError::NotLeader(segment_identity.name()));
        }

        if self
            .cache_manager
            .get_segment_meta(segment_identity)
            .is_none()
        {
            return Err(JournalServerError::SegmentFileMetaNotExists(
                segment_identity.name(),
            ));
        }

        Ok(())
    }
}
