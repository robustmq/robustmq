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
use common_base::tools::now_second;
use protocol::journal_server::journal_engine::{
    JournalEngineError, WriteReqBody, WriteRespMessage, WriteRespMessageStatus,
};
use protocol::journal_server::journal_record::JournalRecord;

use super::file::SegmentFile;
use super::manager::SegmentFileManager;
use crate::core::cache::CacheManager;
use crate::core::error::{get_journal_server_code, JournalServerError};
use crate::index::build_index_message;

pub async fn write_data(
    cache_manager: &Arc<CacheManager>,
    segment_file_manager: &Arc<SegmentFileManager>,
    req_body: &WriteReqBody,
) -> Result<Vec<WriteRespMessage>, JournalServerError> {
    let conf = journal_server_conf();
    let mut results = Vec::new();

    for msg in req_body.data.clone() {
        let segment = if let Some(segment) =
            cache_manager.get_segment(&msg.namespace, &msg.shard_name, msg.segment)
        {
            segment
        } else {
            return Err(JournalServerError::SegmentNotExist(
                msg.shard_name,
                msg.segment,
            ));
        };

        let fold = if let Some(fold) = segment.get_fold(conf.node_id) {
            fold
        } else {
            return Err(JournalServerError::SegmentDataDirectoryNotFound(
                format!("{}-{}", msg.shard_name, msg.segment),
                conf.node_id,
            ));
        };

        let segment = SegmentFile::new(
            msg.namespace.clone(),
            msg.shard_name.clone(),
            msg.segment,
            fold,
        );

        let mut resp_message = WriteRespMessage {
            namespace: msg.namespace.clone(),
            shard_name: msg.shard_name.clone(),
            segment: msg.segment,
            ..Default::default()
        };

        let mut resp_message_status = Vec::new();

        let segment_file = if let Some(segment_file) =
            segment_file_manager.get_segment_file(&msg.namespace, &msg.shard_name, msg.segment)
        {
            segment_file
        } else {
            return Err(JournalServerError::SegmentFileNotExists(format!(
                "{}-{}",
                msg.shard_name, msg.segment
            )));
        };
        let end_offset = segment_file.end_offset;
        for message in msg.messages {
            let record = JournalRecord {
                content: message.value,
                create_time: now_second(),
                key: message.key,
                namespace: msg.namespace.clone(),
                shard_name: msg.shard_name.clone(),
                offset: end_offset,
                segment: msg.segment,
                tags: message.tags,
            };
            let mut status = WriteRespMessageStatus::default();
            match segment.write(record.clone()).await {
                Ok(()) => {
                    status.offset = record.offset;
                    segment_file_manager.incr_end_offset(
                        &msg.namespace,
                        &msg.shard_name,
                        msg.segment,
                    )?;

                    // build index
                    build_index_message()?;
                }
                Err(e) => {
                    status.error = Some(JournalEngineError {
                        code: get_journal_server_code(&e),
                        error: e.to_string(),
                    });
                }
            }
            resp_message_status.push(status);
        }
        resp_message.messages = resp_message_status;
        results.push(resp_message);
    }
    Ok(results)
}
