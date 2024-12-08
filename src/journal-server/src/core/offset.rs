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

use grpc_clients::pool::ClientPool;
use metadata_struct::journal::shard::shard_name_iden;
use protocol::journal_server::journal_engine::AutoOffsetStrategy;
use serde::{Deserialize, Serialize};

use super::cache::CacheManager;
use super::error::JournalServerError;
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;

#[derive(Clone, Serialize, Deserialize)]
pub struct Offset {
    pub namespace: String,
    pub shard_name: String,
    pub offset: u64,
}

pub struct OffsetManager {
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
    segment_file_manager: Arc<SegmentFileManager>,
}

impl OffsetManager {
    pub fn new(
        client_pool: Arc<ClientPool>,
        cache_manager: Arc<CacheManager>,
        segment_file_manager: Arc<SegmentFileManager>,
    ) -> Self {
        OffsetManager {
            client_pool,
            cache_manager,
            segment_file_manager,
        }
    }

    pub async fn get_offset_by_timestamp(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        timestamp: u64,
    ) -> Result<u64, JournalServerError> {
        Ok(0)
    }

    pub async fn get_offset_by_strategy(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
        strategy: AutoOffsetStrategy,
    ) -> Result<u64, JournalServerError> {
        if strategy == AutoOffsetStrategy::Latest {
            return self.get_latest_offset_by_shard(namespace, shard_name).await;
        }
        self.get_earliest_offset_by_shard(namespace, shard_name)
            .await
    }

    async fn get_earliest_offset_by_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Result<u64, JournalServerError> {
        let shard = if let Some(shard) = self.cache_manager.get_shard(namespace, shard_name) {
            shard
        } else {
            return Err(JournalServerError::ShardNotExist(shard_name_iden(
                namespace, shard_name,
            )));
        };

        let start_segment = shard.start_segment_seq;
        let segment_iden = SegmentIdentity {
            namespace: namespace.to_owned(),
            shard_name: shard_name.to_owned(),
            segment_seq: start_segment,
        };
        let segment_meta = if let Some(meta) = self.cache_manager.get_segment_meta(&segment_iden) {
            meta
        } else {
            return Err(JournalServerError::SegmentMetaNotExists(
                segment_iden.name(),
            ));
        };

        Ok(segment_meta.start_offset as u64)
    }

    async fn get_latest_offset_by_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Result<u64, JournalServerError> {
        let shard = if let Some(shard) = self.cache_manager.get_shard(namespace, shard_name) {
            shard
        } else {
            return Err(JournalServerError::ShardNotExist(shard_name_iden(
                namespace, shard_name,
            )));
        };

        let last_segment = shard.last_segment_seq;

        let segment_iden = SegmentIdentity {
            namespace: namespace.to_owned(),
            shard_name: shard_name.to_owned(),
            segment_seq: last_segment,
        };

        let end_offset =
            if let Some(file_meta) = self.segment_file_manager.get_end_offset(&segment_iden) {
                file_meta
            } else {
                return Err(JournalServerError::SegmentFileMetaNotExists(
                    segment_iden.name(),
                ));
            };

        Ok(end_offset as u64)
    }
}
