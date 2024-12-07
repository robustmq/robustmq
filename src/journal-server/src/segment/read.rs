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
    ReadReqBody, ReadReqFilter, ReadReqOptions, ReadRespMessage, ReadRespSegmentMessage, ReadType,
};
use rocksdb_engine::RocksDBEngine;

use super::file::{ReadData, SegmentFile};
use super::SegmentIdentity;
use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;
use crate::index::offset::OffsetIndexManager;
use crate::index::tag::TagIndexManager;
use crate::index::time::TimestampIndexManager;

pub async fn read_data_req(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req_body: &ReadReqBody,
    node_id: u64,
) -> Result<Vec<ReadRespSegmentMessage>, JournalServerError> {
    let mut results = Vec::new();
    for raw in req_body.messages.iter() {
        let mut shard_message = ReadRespSegmentMessage {
            namespace: raw.namespace.to_string(),
            shard_name: raw.shard_name.to_string(),
            segment: raw.segment,
            ..Default::default()
        };

        let segment_iden = SegmentIdentity {
            namespace: raw.namespace.to_string(),
            shard_name: raw.shard_name.to_string(),
            segment_seq: raw.segment,
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

        let segment_file = SegmentFile::new(
            segment_iden.namespace.clone(),
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

        let read_options = if let Some(option) = raw.options.clone() {
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

            ReadType::Timestamp => {
                read_by_timestamp(
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
                offset: record.offset,
                key: record.key,
                value: record.content,
                tags: record.tags,
            });
        }
        shard_message.messages = record_message;

        results.push(shard_message);
    }
    Ok(results)
}

async fn read_by_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    filter: &ReadReqFilter,
    read_options: &ReadReqOptions,
) -> Result<Vec<ReadData>, JournalServerError> {
    let offset_index = OffsetIndexManager::new(rocksdb_engine_handler.clone());
    let start_position = offset_index
        .get_last_nearest_position_by_offset(segment_iden, filter.offset)
        .await?;

    let res = segment_file
        .read_by_offset(start_position, filter.offset, read_options.max_size)
        .await?;

    Ok(res)
}

async fn read_by_timestamp(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    filter: &ReadReqFilter,
    read_options: &ReadReqOptions,
) -> Result<Vec<ReadData>, JournalServerError> {
    let timestamp_index = TimestampIndexManager::new(rocksdb_engine_handler.clone());
    let index_data = timestamp_index
        .get_last_nearest_position_by_timestamp(segment_iden, filter.timestamp)
        .await?;
    let start_position = if let Some(index_data) = index_data {
        index_data.position
    } else {
        0
    };
    segment_file
        .read_by_timestamp(start_position, filter.timestamp, read_options.max_size)
        .await
}

async fn read_by_key(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    filter: &ReadReqFilter,
    read_options: &ReadReqOptions,
) -> Result<Vec<ReadData>, JournalServerError> {
    let tag_index = TagIndexManager::new(rocksdb_engine_handler.clone());
    let index_data_list = tag_index
        .get_last_positions_by_key(
            segment_iden,
            filter.offset,
            filter.key.clone(),
            read_options.max_record,
        )
        .await?;

    let positions = index_data_list.iter().map(|raw| raw.position).collect();
    segment_file
        .read_by_positions(positions, read_options.max_size)
        .await
}

async fn read_by_tag(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file: &SegmentFile,
    segment_iden: &SegmentIdentity,
    filter: &ReadReqFilter,
    read_options: &ReadReqOptions,
) -> Result<Vec<ReadData>, JournalServerError> {
    let tag_index = TagIndexManager::new(rocksdb_engine_handler.clone());
    let index_data_list = tag_index
        .get_last_positions_by_tag(
            segment_iden,
            filter.offset,
            filter.key.clone(),
            read_options.max_record,
        )
        .await?;

    let positions = index_data_list.iter().map(|raw| raw.position).collect();
    segment_file
        .read_by_positions(positions, read_options.max_size)
        .await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn read_base_test() {}
}
