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
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::list_offsets_response::{
    ListOffsetsPartitionResponse, ListOffsetsResponse, ListOffsetsTopicResponse,
};
use kafka_protocol::messages::offset_delete_response::{
    OffsetDeleteResponsePartition, OffsetDeleteResponseTopic,
};
use kafka_protocol::messages::{ListOffsetsRequest, OffsetDeleteRequest, OffsetDeleteResponse};
use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;
use metadata_struct::adapter::adapter_shard::AdapterShardDetail;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

use crate::core::constants::{
    LIST_OFFSETS_EARLIEST_TIMESTAMP, LIST_OFFSETS_LATEST_TIMESTAMP, NO_OFFSET,
};

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

                let offset = resolve_offset_for_partition(
                    p.timestamp,
                    partition,
                    detail,
                    &resolved_by_timestamp,
                );

                ListOffsetsPartitionResponse::default()
                    .with_partition_index(p.partition_index)
                    .with_error_code(0)
                    .with_offset(offset)
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

/// Delete a group's committed offsets for the requested topic-partitions
/// (Kafka OffsetDelete semantics: removes the checkpoint, not the topic data
/// itself — see storage-engine's `delete_records_before` for the latter).
pub async fn process_offset_delete(
    sdm: &Arc<StorageDriverManager>,
    req: &OffsetDeleteRequest,
) -> Option<KafkaPacket> {
    let group_id = req.group_id.to_string();

    // Resolve every requested partition to its shard_name up front, so we can
    // delete every valid shard in a single call and still report a
    // per-partition error code (UnknownTopicOrPartition) for ones that don't
    // resolve. Mirrors process_offset_commit's resolution pattern.
    let mut shard_names = Vec::new();
    let mut partition_resolved: Vec<Vec<bool>> = Vec::with_capacity(req.topics.len());

    for t in &req.topics {
        let topic_name = t.name.to_string();
        let topic = sdm
            .broker_cache
            .get_topic_by_name(get_tenant(), &topic_name);

        let resolved = t
            .partitions
            .iter()
            .map(|p| {
                let Some(shard_name) = topic
                    .as_ref()
                    .and_then(|t| t.storage_name_list.get(&(p.partition_index as u32)))
                else {
                    return false;
                };
                shard_names.push(shard_name.clone());
                true
            })
            .collect();
        partition_resolved.push(resolved);
    }

    let delete_error_code = if shard_names.is_empty() {
        0
    } else if let Err(e) = sdm
        .delete_group_offset(get_tenant(), &group_id, &shard_names)
        .await
    {
        warn!(
            "Kafka OffsetDelete storage error for group {}: {}",
            group_id, e
        );
        ResponseError::UnknownServerError.code()
    } else {
        0
    };

    let topics = req
        .topics
        .iter()
        .zip(partition_resolved)
        .map(|(t, resolved)| {
            let partitions = t
                .partitions
                .iter()
                .zip(resolved)
                .map(|(p, resolved)| {
                    let error_code = if !resolved {
                        ResponseError::UnknownTopicOrPartition.code()
                    } else {
                        delete_error_code
                    };
                    OffsetDeleteResponsePartition::default()
                        .with_partition_index(p.partition_index)
                        .with_error_code(error_code)
                })
                .collect();
            OffsetDeleteResponseTopic::default()
                .with_name(t.name.clone())
                .with_partitions(partitions)
        })
        .collect();

    Some(KafkaPacket::OffsetDeleteResponse(
        OffsetDeleteResponse::default()
            .with_error_code(delete_error_code)
            .with_topics(topics),
    ))
}

/// Pick the offset ListOffsets should report for one partition: the shard's
/// known start/high-watermark for the earliest/latest sentinels, or the
/// pre-resolved timestamp lookup result. For a real timestamp with no message
/// at/after it (e.g. a future timestamp), Kafka returns -1 (`NO_OFFSET`) rather
/// than a real offset, so tools like kafka-get-offsets can tell "no match" from
/// "offset 0".
fn resolve_offset_for_partition(
    timestamp: i64,
    partition: u32,
    detail: &AdapterShardDetail,
    resolved_by_timestamp: &HashMap<i64, HashMap<u32, u64>>,
) -> i64 {
    match timestamp {
        LIST_OFFSETS_EARLIEST_TIMESTAMP => detail.offset.start_offset as i64,
        LIST_OFFSETS_LATEST_TIMESTAMP => detail.offset.high_watermark as i64,
        ts => resolved_by_timestamp
            .get(&ts)
            .and_then(|offsets| offsets.get(&partition))
            .map(|&offset| offset as i64)
            .unwrap_or(NO_OFFSET),
    }
}

fn unknown_partition_response(partition_index: i32) -> ListOffsetsPartitionResponse {
    ListOffsetsPartitionResponse::default()
        .with_partition_index(partition_index)
        .with_error_code(ResponseError::UnknownTopicOrPartition.code())
        .with_offset(NO_OFFSET)
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

    fn make_detail(start_offset: u64, high_watermark: u64) -> AdapterShardDetail {
        AdapterShardDetail {
            shard_name: "shard".to_string(),
            topic_name: "topic".to_string(),
            config: Default::default(),
            shard: Default::default(),
            offset: metadata_struct::adapter::adapter_shard::AdapterShardDetailOffset {
                start_offset,
                end_offset: high_watermark,
                high_watermark,
            },
            desc: String::new(),
        }
    }

    #[test]
    fn resolve_offset_for_partition_picks_by_timestamp_sentinel() {
        let detail = make_detail(10, 100);
        let empty = HashMap::new();

        assert_eq!(
            resolve_offset_for_partition(LIST_OFFSETS_EARLIEST_TIMESTAMP, 0, &detail, &empty),
            10
        );
        assert_eq!(
            resolve_offset_for_partition(LIST_OFFSETS_LATEST_TIMESTAMP, 0, &detail, &empty),
            100
        );

        let mut resolved = HashMap::new();
        resolved.insert(1234i64, HashMap::from([(0u32, 42u64)]));
        assert_eq!(
            resolve_offset_for_partition(1234, 0, &detail, &resolved),
            42
        );
        // Timestamp resolved for other partitions but not this one (no message
        // at/after the timestamp here) reports -1, matching Kafka.
        assert_eq!(
            resolve_offset_for_partition(1234, 1, &detail, &resolved),
            NO_OFFSET
        );
    }
}
