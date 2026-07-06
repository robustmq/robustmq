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
use kafka_protocol::messages::offset_commit_response::{
    OffsetCommitResponsePartition, OffsetCommitResponseTopic,
};
use kafka_protocol::messages::offset_fetch_response::{
    OffsetFetchResponsePartition, OffsetFetchResponseTopic,
};
use kafka_protocol::messages::{
    OffsetCommitRequest, OffsetCommitResponse, OffsetFetchRequest, OffsetFetchResponse, TopicName,
};
use kafka_protocol::protocol::StrBytes;
use metadata_struct::adapter::adapter_offset::AdapterCommitOffset;

use crate::core::constants::NO_OFFSET;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

pub async fn process_offset_commit(
    sdm: &Arc<StorageDriverManager>,
    req: &OffsetCommitRequest,
) -> Option<KafkaPacket> {
    let group_id = req.group_id.to_string();

    // Resolve each requested partition to its shard_name up front, so we can
    // commit every valid shard in a single call and still report a per-partition
    // error code (UnknownTopicOrPartition) for ones that don't resolve.
    let mut commit_offsets = Vec::new();
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
                if p.committed_offset != NO_OFFSET {
                    commit_offsets.push(AdapterCommitOffset {
                        shard_name: shard_name.clone(),
                        topic_name: topic_name.clone(),
                        partition: p.partition_index as u32,
                        offset: p.committed_offset as u64,
                    });
                }
                true
            })
            .collect();
        partition_resolved.push(resolved);
    }

    let commit_error_code = if commit_offsets.is_empty() {
        0
    } else if let Err(e) = sdm
        .commit_group_offset(get_tenant(), &group_id, &commit_offsets)
        .await
    {
        warn!(
            "Kafka OffsetCommit storage error for group {}: {}",
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
                    } else if p.committed_offset == NO_OFFSET {
                        0
                    } else {
                        commit_error_code
                    };
                    OffsetCommitResponsePartition::default()
                        .with_partition_index(p.partition_index)
                        .with_error_code(error_code)
                })
                .collect();
            OffsetCommitResponseTopic::default()
                .with_name(t.name.clone())
                .with_partitions(partitions)
        })
        .collect();

    Some(KafkaPacket::OffsetCommitResponse(
        OffsetCommitResponse::default().with_topics(topics),
    ))
}

struct FetchedPartition {
    partition_index: i32,
    offset: i64,
    error_code: i16,
}

// Fetches every offset the group has ever committed, then either returns it
// all (requested_topics = None, i.e. the client asked for every topic) or
// filters/joins it against the requested (topic, partition) pairs. Either way
// this is exactly one call to get_offset_by_group — group_id is the only real
// key this API has; topic/partition are just a filter over that result.
async fn fetch_group_offsets(
    sdm: &Arc<StorageDriverManager>,
    group_id: &str,
    requested_topics: Option<&[(String, Vec<i32>)]>,
) -> (i16, Vec<(String, Vec<FetchedPartition>)>) {
    let committed = match sdm.get_offset_by_group(get_tenant(), group_id).await {
        Ok(offsets) => offsets,
        Err(e) => {
            warn!(
                "Kafka OffsetFetch storage error for group {}: {}",
                group_id, e
            );
            return (ResponseError::UnknownServerError.code(), Vec::new());
        }
    };

    let Some(requested_topics) = requested_topics else {
        let mut by_topic: HashMap<String, Vec<FetchedPartition>> = HashMap::new();
        for o in &committed {
            by_topic
                .entry(o.topic_name.clone())
                .or_default()
                .push(FetchedPartition {
                    partition_index: o.partition as i32,
                    offset: o.offset as i64,
                    error_code: 0,
                });
        }
        return (0, by_topic.into_iter().collect());
    };

    let committed_map: HashMap<(&str, i32), i64> = committed
        .iter()
        .map(|o| ((o.topic_name.as_str(), o.partition as i32), o.offset as i64))
        .collect();

    let mut result = Vec::with_capacity(requested_topics.len());
    for (topic_name, partitions) in requested_topics {
        let topic = sdm.broker_cache.get_topic_by_name(get_tenant(), topic_name);

        let fetched = partitions
            .iter()
            .map(|&partition_index| {
                if let Some(&offset) = committed_map.get(&(topic_name.as_str(), partition_index)) {
                    return FetchedPartition {
                        partition_index,
                        offset,
                        error_code: 0,
                    };
                }
                // Never committed is not an error; only a truly unknown
                // topic/partition gets UnknownTopicOrPartition.
                let exists = topic
                    .as_ref()
                    .is_some_and(|t| t.storage_name_list.contains_key(&(partition_index as u32)));
                FetchedPartition {
                    partition_index,
                    offset: NO_OFFSET,
                    error_code: if exists {
                        0
                    } else {
                        ResponseError::UnknownTopicOrPartition.code()
                    },
                }
            })
            .collect();
        result.push((topic_name.clone(), fetched));
    }
    (0, result)
}

fn to_requested_topics<T>(
    topics: &Option<Vec<T>>,
    name_of: impl Fn(&T) -> String,
    partitions_of: impl Fn(&T) -> Vec<i32>,
) -> Option<Vec<(String, Vec<i32>)>> {
    topics.as_ref().map(|list| {
        list.iter()
            .map(|t| (name_of(t), partitions_of(t)))
            .collect()
    })
}

pub async fn process_offset_fetch(
    sdm: &Arc<StorageDriverManager>,
    req: &OffsetFetchRequest,
) -> Option<KafkaPacket> {
    use kafka_protocol::messages::offset_fetch_response::{
        OffsetFetchResponseGroup, OffsetFetchResponsePartitions, OffsetFetchResponseTopics,
    };

    // v8+ uses `groups` field; older versions use `topics` field directly.
    if !req.groups.is_empty() {
        let mut groups = Vec::with_capacity(req.groups.len());
        for g in &req.groups {
            let group_id = g.group_id.to_string();
            let requested = to_requested_topics(
                &g.topics,
                |t| t.name.to_string(),
                |t| t.partition_indexes.clone(),
            );
            let (group_error_code, topic_offsets) =
                fetch_group_offsets(sdm, &group_id, requested.as_deref()).await;

            let topics = topic_offsets
                .into_iter()
                .map(|(topic_name, partitions)| {
                    let partitions = partitions
                        .into_iter()
                        .map(|p| {
                            OffsetFetchResponsePartitions::default()
                                .with_partition_index(p.partition_index)
                                .with_committed_offset(p.offset)
                                .with_error_code(p.error_code)
                        })
                        .collect();
                    OffsetFetchResponseTopics::default()
                        .with_name(TopicName(StrBytes::from(topic_name)))
                        .with_partitions(partitions)
                })
                .collect();

            groups.push(
                OffsetFetchResponseGroup::default()
                    .with_group_id(g.group_id.clone())
                    .with_topics(topics)
                    .with_error_code(group_error_code),
            );
        }

        return Some(KafkaPacket::OffsetFetchResponse(
            OffsetFetchResponse::default().with_groups(groups),
        ));
    }

    // Old format: topics directly on request
    let group_id = req.group_id.to_string();
    let requested = to_requested_topics(
        &req.topics,
        |t| t.name.to_string(),
        |t| t.partition_indexes.clone(),
    );
    let (error_code, topic_offsets) =
        fetch_group_offsets(sdm, &group_id, requested.as_deref()).await;

    let topics = topic_offsets
        .into_iter()
        .map(|(topic_name, partitions)| {
            let partitions = partitions
                .into_iter()
                .map(|p| {
                    OffsetFetchResponsePartition::default()
                        .with_partition_index(p.partition_index)
                        .with_committed_offset(p.offset)
                        .with_error_code(p.error_code)
                })
                .collect();
            OffsetFetchResponseTopic::default()
                .with_name(TopicName(StrBytes::from(topic_name)))
                .with_partitions(partitions)
        })
        .collect();

    Some(KafkaPacket::OffsetFetchResponse(
        OffsetFetchResponse::default()
            .with_topics(topics)
            .with_error_code(error_code),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeTopic {
        name: String,
        partitions: Vec<i32>,
    }

    #[test]
    fn to_requested_topics_maps_some_and_none() {
        let topics = Some(vec![FakeTopic {
            name: "t1".to_string(),
            partitions: vec![0, 1],
        }]);
        let result = to_requested_topics(&topics, |t| t.name.clone(), |t| t.partitions.clone());
        assert_eq!(result, Some(vec![("t1".to_string(), vec![0, 1])]));

        let none_topics: Option<Vec<FakeTopic>> = None;
        let result =
            to_requested_topics(&none_topics, |t| t.name.clone(), |t| t.partitions.clone());
        assert_eq!(result, None);
    }
}
