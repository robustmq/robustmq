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

use protocol::journal_server::journal_engine::{
    ReadReqBody, ReadRespMessage, ReadRespSegmentMessage,
};

use super::file::SegmentFile;
use super::SegmentIdentity;
use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;

pub async fn read_data(
    cache_manager: &Arc<CacheManager>,
    req_body: &ReadReqBody,
    node_id: u64,
) -> Result<Vec<ReadRespSegmentMessage>, JournalServerError> {
    let mut results = Vec::new();
    for raw in req_body.messages.iter() {
        let mut shard_message = ReadRespSegmentMessage {
            namespace: raw.namespace.to_string(),
            shard_name: raw.shard_name.to_string(),
            ..Default::default()
        };

        for segment_row in raw.segments.iter() {
            let segment_iden = SegmentIdentity {
                namespace: raw.namespace.to_string(),
                shard_name: raw.shard_name.to_string(),
                segment_seq: segment_row.segment,
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
            let segment = SegmentFile::new(
                segment_iden.namespace,
                segment_iden.shard_name,
                segment_iden.segment_seq,
                fold,
            );
            let res = segment.read(Some(segment_row.offset), 5).await?;
            shard_message.segment = segment_iden.segment_seq;

            let mut record_message = Vec::new();
            for read_data in res {
                let record = read_data.record;
                record_message.push(ReadRespMessage {
                    offset: record.offset,
                    key: record.key,
                    value: record.content,
                    tags: record.tags,
                });
            }
            shard_message.messages = record_message;
        }
        results.push(shard_message);
    }
    Ok(results)
}
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn read_base_test() {}
}
