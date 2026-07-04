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

use std::collections::HashMap;
use std::sync::Arc;

use crate::handler::tenant::get_tenant;
use dashmap::DashMap;
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::fetch_response::{FetchableTopicResponse, PartitionData};
use kafka_protocol::messages::list_offsets_response::{
    ListOffsetsPartitionResponse, ListOffsetsResponse, ListOffsetsTopicResponse,
};
use kafka_protocol::messages::{FetchRequest, FetchResponse, ListOffsetsRequest, ProduceRequest};
use kafka_protocol::records::{
    Compression, Record, RecordBatchEncoder, RecordEncodeOptions, TimestampType,
};
use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::adapter::adapter_shard::AdapterShardDetail;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

use crate::core::constants::{
    LIST_OFFSETS_EARLIEST_TIMESTAMP, LIST_OFFSETS_LATEST_TIMESTAMP, NO_LAST_STABLE_OFFSET,
    NO_OFFSET, NO_PRODUCER_EPOCH, NO_PRODUCER_ID,
};

pub type ShardOffsets = Arc<DashMap<(u64, String), HashMap<String, u64>>>;

pub fn process_produce(_req: &ProduceRequest) -> Option<KafkaPacket> {
    None
}

pub async fn process_fetch(
    sdm: &Arc<StorageDriverManager>,
    shard_offsets: &ShardOffsets,
    req: &FetchRequest,
    connection_id: u64,
) -> Option<KafkaPacket> {
    let read_config = AdapterReadConfig::new();
    let mut topic_responses = Vec::new();

    for fetch_topic in &req.topics {
        let topic_name = fetch_topic.topic.to_string();
        let key = (connection_id, topic_name.clone());

        let mut offsets = shard_offsets
            .get(&key)
            .map(|r| r.clone())
            .unwrap_or_default();

        let records_bytes = match sdm
            .read_by_offset(get_tenant(), &topic_name, &offsets, &read_config)
            .await
        {
            Ok(records) if records.is_empty() => None,
            Ok(records) => {
                let mut kafka_records = Vec::new();
                for (i, record) in records.iter().enumerate() {
                    offsets.insert(record.metadata.shard.clone(), record.metadata.offset + 1);
                    kafka_records.push(Record {
                        transactional: false,
                        control: false,
                        partition_leader_epoch: 0,
                        producer_id: NO_PRODUCER_ID,
                        producer_epoch: NO_PRODUCER_EPOCH,
                        timestamp_type: TimestampType::Creation,
                        offset: record.metadata.offset as i64,
                        sequence: i as i32,
                        timestamp: 0,
                        key: None,
                        value: Some(record.data.clone()),
                        headers: Default::default(),
                    });
                }
                shard_offsets.insert(key, offsets);

                let mut buf = bytes::BytesMut::new();
                let opts = RecordEncodeOptions {
                    version: 2,
                    compression: Compression::None,
                };
                RecordBatchEncoder::encode(&mut buf, kafka_records.iter(), &opts).ok()?;
                Some(buf.freeze())
            }
            Err(e) => {
                warn!("Kafka Fetch storage error for {}: {}", topic_name, e);
                None
            }
        };

        let mut partition_responses = Vec::new();
        for fetch_partition in &fetch_topic.partitions {
            partition_responses.push(
                PartitionData::default()
                    .with_partition_index(fetch_partition.partition)
                    .with_error_code(0)
                    .with_high_watermark(i64::MAX)
                    .with_last_stable_offset(NO_LAST_STABLE_OFFSET)
                    .with_log_start_offset(0)
                    .with_records(records_bytes.clone()),
            );
        }

        topic_responses.push(
            FetchableTopicResponse::default()
                .with_topic(fetch_topic.topic.clone())
                .with_partitions(partition_responses),
        );
    }

    let resp = FetchResponse::default()
        .with_error_code(0)
        .with_session_id(0)
        .with_responses(topic_responses);

    Some(KafkaPacket::FetchResponse(resp))
}

fn unknown_partition_response(partition_index: i32) -> ListOffsetsPartitionResponse {
    ListOffsetsPartitionResponse::default()
        .with_partition_index(partition_index)
        .with_error_code(ResponseError::UnknownTopicOrPartition.code())
        .with_offset(NO_OFFSET)
}

pub async fn process_list_offsets(
    sdm: &Arc<StorageDriverManager>,
    req: &ListOffsetsRequest,
) -> Option<KafkaPacket> {
    let mut topic_responses = Vec::with_capacity(req.topics.len());
    for topic_req in &req.topics {
        let topic_name = topic_req.name.to_string();

        let details: HashMap<u32, AdapterShardDetail> =
            match sdm.list_storage_resource(get_tenant(), &topic_name).await {
                Ok(details) => details,
                Err(e) => {
                    warn!("Kafka ListOffsets storage error for {}: {}", topic_name, e);
                    let partitions = topic_req
                        .partitions
                        .iter()
                        .map(|p| unknown_partition_response(p.partition_index))
                        .collect();
                    topic_responses.push(
                        ListOffsetsTopicResponse::default()
                            .with_name(topic_req.name.clone())
                            .with_partitions(partitions),
                    );
                    continue;
                }
            };

        // Real (non-sentinel) timestamps are resolved once per distinct value:
        // get_offset_by_timestamp scans every shard of the topic per call, so
        // partitions that share a timestamp (the common case) share one call.
        let mut resolved_by_timestamp: HashMap<i64, HashMap<u32, u64>> = HashMap::new();
        for ts in topic_req
            .partitions
            .iter()
            .map(|p| p.timestamp)
            .filter(|&ts| {
                ts != LIST_OFFSETS_EARLIEST_TIMESTAMP && ts != LIST_OFFSETS_LATEST_TIMESTAMP
            })
        {
            if resolved_by_timestamp.contains_key(&ts) {
                continue;
            }
            let offsets = sdm
                .get_offset_by_timestamp(
                    get_tenant(),
                    &topic_name,
                    ts as u64,
                    AdapterOffsetStrategy::Earliest,
                )
                .await
                .unwrap_or_else(|e| {
                    warn!(
                        "Kafka ListOffsets timestamp lookup failed for {} at ts={}: {}",
                        topic_name, ts, e
                    );
                    HashMap::new()
                });
            resolved_by_timestamp.insert(ts, offsets);
        }

        let partitions = topic_req
            .partitions
            .iter()
            .map(|p| {
                let partition = p.partition_index as u32;
                let Some(detail) = details.get(&partition) else {
                    return unknown_partition_response(p.partition_index);
                };

                let offset = match p.timestamp {
                    LIST_OFFSETS_EARLIEST_TIMESTAMP => detail.offset.start_offset,
                    LIST_OFFSETS_LATEST_TIMESTAMP => detail.offset.high_watermark,
                    ts => resolved_by_timestamp
                        .get(&ts)
                        .and_then(|offsets| offsets.get(&partition))
                        .copied()
                        .unwrap_or(detail.offset.start_offset),
                };

                ListOffsetsPartitionResponse::default()
                    .with_partition_index(p.partition_index)
                    .with_error_code(0)
                    .with_offset(offset as i64)
            })
            .collect();

        topic_responses.push(
            ListOffsetsTopicResponse::default()
                .with_name(topic_req.name.clone())
                .with_partitions(partitions),
        );
    }

    Some(KafkaPacket::ListOffsetsResponse(
        ListOffsetsResponse::default().with_topics(topic_responses),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_partition_response_sets_error_and_negative_offset() {
        let resp = unknown_partition_response(3);
        assert_eq!(resp.partition_index, 3);
        assert_eq!(
            resp.error_code,
            ResponseError::UnknownTopicOrPartition.code()
        );
        assert_eq!(resp.offset, NO_OFFSET);
    }
}
