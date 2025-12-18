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

use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use protocol::storage::storage_engine_engine::{
    ReadReq, ReadRespSegmentMessage, WriteReq, WriteRespMessage,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::core::shard::try_auto_create_shard;
use crate::segment::storage::manager::SegmentFileManager;
use crate::segment::storage::read::read_data_req;
use crate::segment::storage::write::write_data_req;
use crate::segment::storage::SegmentIdentity;

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
        if self
            .cache_manager
            .get_shard(&segment_identity.shard_name)
            .is_none()
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
