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

use common_base::config::journal_server::journal_server_conf;
use grpc_clients::pool::ClientPool;
use protocol::journal_server::journal_engine::{
    AutoOffsetStrategy, FetchOffsetReq, FetchOffsetRespBody, FetchOffsetShard,
    FetchOffsetShardMeta, ReadReq, ReadRespSegmentMessage, WriteReq, WriteRespMessage,
};
use rocksdb_engine::RocksDBEngine;

use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;
use crate::core::offset::OffsetManager;
use crate::index::time::TimestampIndexManager;
use crate::segment::manager::SegmentFileManager;
use crate::segment::read::read_data_req;
use crate::segment::write::write_data_req;
use crate::segment::SegmentIdentity;

#[derive(Clone)]
pub struct DataHandler {
    cache_manager: Arc<CacheManager>,
    offset_manager: Arc<OffsetManager>,
    segment_file_manager: Arc<SegmentFileManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_pool: Arc<ClientPool>,
}

impl DataHandler {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        offset_manager: Arc<OffsetManager>,
        segment_file_manager: Arc<SegmentFileManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_pool: Arc<ClientPool>,
    ) -> DataHandler {
        DataHandler {
            cache_manager,
            offset_manager,
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
            let segment_identity = SegmentIdentity {
                namespace: message.namespace.to_string(),
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
        let conf = journal_server_conf();
        for row in req_body.messages.clone() {
            let segment_identity = SegmentIdentity {
                namespace: row.namespace.to_string(),
                shard_name: row.shard_name.to_string(),
                segment_seq: row.segment,
            };
            self.validator(&segment_identity)?;
        }

        let conf = journal_server_conf();
        let results = read_data_req(
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            &req_body,
            conf.node_id,
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
        let group_name = req_body.group_name.clone();
        let strategy = req_body.auto_offset_strategy();
        let mut meta_list = Vec::new();
        for shard in req_body.shards {
            let offset = self.get_offset_by_timestamp(&shard, strategy).await?;
            let meta = FetchOffsetShardMeta {
                namespace: shard.namespace,
                shard_name: shard.shard_name,
                offset,
            };

            meta_list.push(meta);
        }

        Ok(FetchOffsetRespBody {
            group_name,
            shard_offsets: meta_list,
        })
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &FetchOffsetShard,
        strategy: AutoOffsetStrategy,
    ) -> Result<u64, JournalServerError> {
        let conf = journal_server_conf();
        let segment_iden = SegmentIdentity {
            namespace: shard.namespace.to_owned(),
            shard_name: shard.shard_name.to_owned(),
            segment_seq: shard.segment_no,
        };
        let timestamp_index = TimestampIndexManager::new(self.rocksdb_engine_handler.clone());
        let offset = if let Some(index_data) = timestamp_index
            .get_last_nearest_position_by_timestamp(&segment_iden, shard.timestamp)
            .await?
        {
            index_data.offset
        } else {
            self.offset_manager
                .get_offset_by_strategy(
                    &conf.cluster_name,
                    &shard.namespace,
                    &shard.shard_name,
                    strategy,
                )
                .await?
        };
        Ok(offset)
    }

    fn validator(&self, segment_identity: &SegmentIdentity) -> Result<(), JournalServerError> {
        if self
            .cache_manager
            .get_shard(&segment_identity.namespace, &segment_identity.shard_name)
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

        let conf = journal_server_conf();
        if segment.leader != conf.node_id {
            return Err(JournalServerError::NotLeader(segment_identity.name()));
        }

        Ok(())
    }
}
