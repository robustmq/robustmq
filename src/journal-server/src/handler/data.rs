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
    JournalEngineError, OffsetCommitReq, OffsetCommitShardResp, ReadReq, ReadRespSegmentMessage,
    WriteReq, WriteRespMessage,
};

use crate::core::cache::CacheManager;
use crate::core::error::{get_journal_server_code, JournalServerError};
use crate::core::offset::OffsetManager;
use crate::segment::manager::SegmentFileManager;
use crate::segment::read::read_data;
use crate::segment::write::write_data;

#[derive(Clone)]
pub struct DataHandler {
    cache_manager: Arc<CacheManager>,
    offset_manager: Arc<OffsetManager>,
    segment_file_manager: Arc<SegmentFileManager>,
}

impl DataHandler {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        offset_manager: Arc<OffsetManager>,
        segment_file_manager: Arc<SegmentFileManager>,
    ) -> DataHandler {
        DataHandler {
            cache_manager,
            offset_manager,
            segment_file_manager,
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
        for message in req_body.data.clone() {
            self.valitator(&message.namespace, &message.shard_name, message.segment)?;
        }

        let results =
            write_data(&self.cache_manager, &self.segment_file_manager, &req_body).await?;
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
            for segment in row.segments {
                self.valitator(&row.namespace, &row.shard_name, segment.segment)?;
            }
        }

        let conf = journal_server_conf();
        let results = read_data(&self.cache_manager, &req_body, conf.node_id).await?;
        Ok(results)
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
                let e = JournalServerError::ShardNotExist(shard.shard_name.clone());
                result.push(OffsetCommitShardResp {
                    shard_name: shard.shard_name.clone(),
                    error: Some(JournalEngineError {
                        code: get_journal_server_code(&e),
                        error: e.to_string(),
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

    fn valitator(
        &self,
        namespace: &str,
        shard_name: &str,
        segment_no: u32,
    ) -> Result<(), JournalServerError> {
        if self
            .cache_manager
            .get_shard(namespace, shard_name)
            .is_none()
        {
            return Err(JournalServerError::ShardNotExist(shard_name.to_string()));
        }

        let segment = if let Some(segment) = self
            .cache_manager
            .get_segment(namespace, shard_name, segment_no)
        {
            segment
        } else {
            return Err(JournalServerError::SegmentNotExist(
                shard_name.to_string(),
                segment_no,
            ));
        };

        if !segment.allow_read() {
            return Err(JournalServerError::SegmentStatusError(
                shard_name.to_string(),
                format!("{:?}", segment.status),
            ));
        }

        let conf = journal_server_conf();
        if segment.leader != conf.node_id {
            return Err(JournalServerError::NotLeader(
                shard_name.to_string(),
                segment_no,
            ));
        }
        Ok(())
    }
}
