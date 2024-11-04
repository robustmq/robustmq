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
use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;

pub async fn read_data(
    cache_manager: &Arc<CacheManager>,
    req_body: &ReadReqBody,
    node_id: u64,
) -> Result<Vec<ReadRespSegmentMessage>, JournalServerError> {
    let mut results = Vec::new();
    for raw in req_body.messages.clone() {
        let namespace = raw.namespace;
        let shard_name = raw.shard_name;
        let mut shard_message = ReadRespSegmentMessage {
            namespace: namespace.clone(),
            shard_name: shard_name.clone(),
            ..Default::default()
        };
        for segment_row in raw.segments {
            let segment_no = segment_row.segment;
            let segment = if let Some(segment) =
                cache_manager.get_segment(&namespace, &shard_name, segment_no)
            {
                segment
            } else {
                return Err(JournalServerError::SegmentNotExist(shard_name, segment_no));
            };

            let fold = if let Some(fold) = segment.get_fold(node_id) {
                fold
            } else {
                return Err(JournalServerError::SegmentDataDirectoryNotFound(
                    format!("{}-{}", shard_name, segment_no),
                    node_id,
                ));
            };
            let segment = SegmentFile::new(namespace.clone(), shard_name.clone(), segment_no, fold);
            let res = segment.read(Some(segment_row.offset), 5).await?;
            shard_message.segment = segment_no;

            let mut record_message = Vec::new();
            for record in res {
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
