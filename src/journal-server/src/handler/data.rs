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
use protocol::journal_server::journal_engine::{
    JournalEngineError, OffsetCommitReq, OffsetCommitShardResp, ReadReq, ReadRespMessage, WriteReq,
    WriteRespMessage,
};

use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;
use crate::core::offset::OffsetManager;

#[derive(Clone)]
pub struct DataHandler {
    cache_manager: Arc<CacheManager>,
    offset_manager: Arc<OffsetManager>,
}

impl DataHandler {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        offset_manager: Arc<OffsetManager>,
    ) -> DataHandler {
        DataHandler {
            cache_manager,
            offset_manager,
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
        let conf = journal_server_conf();
        for message in req_body.messages {
            if self
                .cache_manager
                .get_shard(&message.namespace, &message.shard_name)
                .is_none()
            {
                return Err(JournalServerError::ShardNotExist(message.shard_name));
            }

            let segment = if let Some(segment) = self.cache_manager.get_segment(
                &message.namespace,
                &message.shard_name,
                message.segment,
            ) {
                segment
            } else {
                return Err(JournalServerError::SegmentNotExist(
                    message.shard_name,
                    message.segment,
                ));
            };

            if segment.is_seal_up() {
                return Err(JournalServerError::SegmentHasBeenSealed(
                    message.shard_name,
                    message.segment,
                ));
            }

            if segment.leader != conf.node_id {
                return Err(JournalServerError::NotLeader(
                    message.shard_name,
                    message.segment,
                ));
            }
        }

        Ok(Vec::new())
    }

    pub async fn read(&self, request: ReadReq) -> Result<Vec<ReadRespMessage>, JournalServerError> {
        Ok(Vec::new())
    }

    pub async fn offset_commit(
        &self,
        request: OffsetCommitReq,
    ) -> Result<Vec<OffsetCommitShardResp>, JournalServerError> {
        if request.body.is_none() {
            return Err(JournalServerError::RequestBodyNotEmpty(
                "offset_commit".to_string(),
            ));
        }

        let req_body = request.body.unwrap();
        let mut result = Vec::new();
        for shard in req_body.shard {
            if self
                .cache_manager
                .get_shard(&req_body.namespace, &shard.shard_name)
                .is_none()
            {
                result.push(OffsetCommitShardResp {
                    shard_name: shard.shard_name.clone(),
                    error: Some(JournalEngineError {
                        code: 1,
                        error: JournalServerError::ShardNotExist(shard.shard_name).to_string(),
                    }),
                });
                continue;
            }
            let conf = journal_server_conf();

            self.offset_manager
                .commit_offset(
                    &conf.cluster_name,
                    &req_body.namespace,
                    &shard.shard_name,
                    shard.offset,
                )
                .await?;

            result.push(OffsetCommitShardResp {
                shard_name: shard.shard_name.clone(),
                ..Default::default()
            });
        }
        Ok(result)
    }
}
