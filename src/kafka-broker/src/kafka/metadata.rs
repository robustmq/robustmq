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

use std::ops::Range;
use std::sync::Arc;

use crate::core::consumer_group_meta::topic_uuid;
use crate::core::coordinator_locator::split_host_port;
use crate::handler::tenant::get_tenant;
use broker_core::cache::NodeCacheManager;
use common_config::broker::broker_config;
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::describe_cluster_response::DescribeClusterBroker;
use kafka_protocol::messages::describe_topic_partitions_response::{
    Cursor, DescribeTopicPartitionsResponsePartition, DescribeTopicPartitionsResponseTopic,
};
use kafka_protocol::messages::metadata_response::{
    MetadataResponseBroker, MetadataResponsePartition, MetadataResponseTopic,
};
use kafka_protocol::messages::{
    DescribeClusterRequest, DescribeClusterResponse, DescribeTopicPartitionsRequest,
    DescribeTopicPartitionsResponse, MetadataRequest, MetadataResponse, TopicName,
};
use kafka_protocol::protocol::StrBytes;
use metadata_struct::topic::Topic;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::StorageDriverManager;

// RobustMQ has no KRaft-style separate controller listener, so DescribeCluster
// always describes the broker endpoint regardless of what the request asked for.
const ENDPOINT_TYPE_BROKERS: i8 = 1;

// ACLs aren't enforced yet, so report every operation as authorized rather
// than fabricating a bitmask we can't back up.
const ALL_OPERATIONS_AUTHORIZED: i32 = -1;

pub fn process_metadata(
    broker_cache: &Arc<NodeCacheManager>,
    sdm: &Arc<StorageDriverManager>,
    req: &MetadataRequest,
) -> Option<KafkaPacket> {
    let topics = build_topics_from_cache(broker_cache, sdm, req);
    let brokers = build_brokers_from_cache(broker_cache);
    let controller_id = pick_controller_id(broker_cache);

    let resp = MetadataResponse::default()
        .with_brokers(brokers)
        .with_controller_id(controller_id.into())
        .with_topics(topics);

    Some(KafkaPacket::MetadataResponse(resp))
}

fn build_brokers_from_cache(cache: &Arc<NodeCacheManager>) -> Vec<MetadataResponseBroker> {
    cache
        .node_list()
        .into_iter()
        .filter_map(|node| {
            let (host, port) = split_host_port(&node.extend.kafka.tcp_addr)?;
            Some(
                MetadataResponseBroker::default()
                    .with_node_id((node.node_id as i32).into())
                    .with_host(StrBytes::from(host))
                    .with_port(port),
            )
        })
        .collect()
}

pub fn process_describe_cluster(
    broker_cache: &Arc<NodeCacheManager>,
    _req: &DescribeClusterRequest,
) -> Option<KafkaPacket> {
    let brokers = build_cluster_brokers_from_cache(broker_cache);
    let controller_id = pick_controller_id(broker_cache);

    let resp = DescribeClusterResponse::default()
        .with_endpoint_type(ENDPOINT_TYPE_BROKERS)
        .with_cluster_id(StrBytes::from(broker_config().cluster_name.clone()))
        .with_controller_id(controller_id.into())
        .with_brokers(brokers)
        .with_cluster_authorized_operations(ALL_OPERATIONS_AUTHORIZED);

    Some(KafkaPacket::DescribeClusterResponse(resp))
}

// RobustMQ tracks neither rack placement nor a KRaft-style fenced state, so
// both fields fall back to their "unset"/"healthy" defaults for every node.
fn build_cluster_brokers_from_cache(cache: &Arc<NodeCacheManager>) -> Vec<DescribeClusterBroker> {
    cache
        .node_list()
        .into_iter()
        .filter_map(|node| {
            let (host, port) = split_host_port(&node.extend.kafka.tcp_addr)?;
            Some(
                DescribeClusterBroker::default()
                    .with_broker_id((node.node_id as i32).into())
                    .with_host(StrBytes::from(host))
                    .with_port(port)
                    .with_rack(None)
                    .with_is_fenced(false),
            )
        })
        .collect()
}

// todo
fn pick_controller_id(cache: &Arc<NodeCacheManager>) -> i32 {
    cache
        .node_list()
        .into_iter()
        .map(|n| n.node_id as i32)
        .min()
        .unwrap_or(0)
}

fn build_topics_from_cache(
    cache: &Arc<NodeCacheManager>,
    sdm: &Arc<StorageDriverManager>,
    req: &MetadataRequest,
) -> Vec<MetadataResponseTopic> {
    let requested = req.topics.as_deref().unwrap_or(&[]);

    if requested.is_empty() {
        return cache
            .list_topics_by_tenant(get_tenant())
            .into_iter()
            .map(|topic| topic_to_metadata(topic, sdm))
            .collect();
    }

    requested
        .iter()
        .filter_map(|t| t.name.clone())
        .map(|name| match cache.get_topic_by_name(get_tenant(), &name) {
            Some(topic) => topic_to_metadata(topic, sdm),
            None => MetadataResponseTopic::default()
                .with_error_code(ResponseError::UnknownTopicOrPartition.code())
                .with_name(Some(name))
                .with_is_internal(false)
                .with_partitions(vec![]),
        })
        .collect()
}

fn topic_to_metadata(topic: Topic, sdm: &Arc<StorageDriverManager>) -> MetadataResponseTopic {
    let partitions = (0..topic.partition.max(1))
        .map(|i| partition_metadata(i as i32, &topic, sdm))
        .collect();
    MetadataResponseTopic::default()
        .with_error_code(0)
        .with_topic_id(topic_uuid(get_tenant(), &topic.topic_name))
        .with_name(Some(TopicName(StrBytes::from(topic.topic_name))))
        .with_is_internal(false)
        .with_partitions(partitions)
}

struct PartitionReplicaState {
    leader_id: i32,
    leader_epoch: i32,
    replica_nodes: Vec<i32>,
    isr_nodes: Vec<i32>,
}

// Partition leader/replicas/ISR are read from the shard's active segment
// (owned by storage-engine's ISR/leader-rebalance machinery) rather than
// tracked separately here, so failover and rebalancing stay in sync
// automatically. Falls back to broker 0 when the shard has no active
// segment yet (e.g. topic storage type without ISR, or not yet created).
fn partition_replica_state(
    partition_index: i32,
    topic: &Topic,
    sdm: &Arc<StorageDriverManager>,
) -> PartitionReplicaState {
    let segment = topic
        .storage_name_list
        .get(&(partition_index as u32))
        .and_then(|shard_name| {
            sdm.engine_storage_handler
                .cache_manager
                .get_active_segment(shard_name)
        });

    match segment {
        Some(segment) => PartitionReplicaState {
            leader_id: segment.leader as i32,
            leader_epoch: segment.leader_epoch as i32,
            replica_nodes: segment.replicas.iter().map(|r| r.node_id as i32).collect(),
            isr_nodes: segment.isr.iter().map(|&node_id| node_id as i32).collect(),
        },
        None => PartitionReplicaState {
            leader_id: 0,
            leader_epoch: -1,
            replica_nodes: vec![0],
            isr_nodes: vec![0],
        },
    }
}

fn partition_metadata(
    partition_index: i32,
    topic: &Topic,
    sdm: &Arc<StorageDriverManager>,
) -> MetadataResponsePartition {
    let state = partition_replica_state(partition_index, topic, sdm);
    MetadataResponsePartition::default()
        .with_error_code(0)
        .with_partition_index(partition_index)
        .with_leader_id(state.leader_id.into())
        .with_replica_nodes(state.replica_nodes.into_iter().map(Into::into).collect())
        .with_isr_nodes(state.isr_nodes.into_iter().map(Into::into).collect())
}

fn describe_topic_partition(
    partition_index: i32,
    topic: &Topic,
    sdm: &Arc<StorageDriverManager>,
) -> DescribeTopicPartitionsResponsePartition {
    let state = partition_replica_state(partition_index, topic, sdm);
    DescribeTopicPartitionsResponsePartition::default()
        .with_error_code(0)
        .with_partition_index(partition_index)
        .with_leader_id(state.leader_id.into())
        .with_leader_epoch(state.leader_epoch)
        .with_replica_nodes(state.replica_nodes.into_iter().map(Into::into).collect())
        .with_isr_nodes(state.isr_nodes.into_iter().map(Into::into).collect())
        .with_offline_replicas(vec![])
}

struct TopicPartitionCount {
    name: String,
    partition_count: u32,
}

type TopicCursor = (String, i32);
type TopicPage = (usize, Range<u32>);

/// Decide which (topic, partition-range) slices belong on this page and
/// where the next page should resume, per the DescribeTopicPartitions
/// cursor protocol (KIP-966): topics are walked in name order (`topics`
/// must already be sorted), partitions within a topic in index order, and
/// the page stops once `limit` partitions have been emitted. A topic with
/// `partition_count == 0` (unknown topic, reported as an error entry with
/// no partitions) never consumes budget, so it's always included as soon
/// as the walk reaches it.
fn paginate_topic_partitions(
    topics: &[TopicPartitionCount],
    cursor: Option<TopicCursor>,
    limit: usize,
) -> (Vec<TopicPage>, Option<TopicCursor>) {
    let start_topic = cursor
        .as_ref()
        .and_then(|(name, _)| topics.iter().position(|t| &t.name == name))
        .unwrap_or(0);
    let start_partition = cursor.map(|(_, p)| p.max(0) as u32).unwrap_or(0);

    let mut pages = Vec::new();
    let mut budget = limit;
    for (topic_idx, topic) in topics.iter().enumerate().skip(start_topic) {
        let from = if topic_idx == start_topic {
            start_partition.min(topic.partition_count)
        } else {
            0
        };
        let available = (topic.partition_count - from) as usize;
        if available <= budget {
            pages.push((topic_idx, from..topic.partition_count));
            budget -= available;
        } else {
            let cut = from + budget as u32;
            if cut > from {
                pages.push((topic_idx, from..cut));
            }
            return (pages, Some((topic.name.clone(), cut as i32)));
        }
    }
    (pages, None)
}

fn effective_partition_limit(req: &DescribeTopicPartitionsRequest) -> usize {
    let cap = broker_config().kafka_runtime.max_describe_topic_partitions as usize;
    let requested = if req.response_partition_limit > 0 {
        req.response_partition_limit as usize
    } else {
        cap
    };
    requested.min(cap)
}

pub fn process_describe_topic_partitions(
    broker_cache: &Arc<NodeCacheManager>,
    sdm: &Arc<StorageDriverManager>,
    req: &DescribeTopicPartitionsRequest,
) -> Option<KafkaPacket> {
    let mut names: Vec<String> = req.topics.iter().map(|t| t.name.to_string()).collect();
    names.sort();

    let topics: Vec<(String, Option<Topic>)> = names
        .into_iter()
        .map(|name| {
            let topic = broker_cache.get_topic_by_name(get_tenant(), &name);
            (name, topic)
        })
        .collect();

    let counts: Vec<TopicPartitionCount> = topics
        .iter()
        .map(|(name, topic)| TopicPartitionCount {
            name: name.clone(),
            partition_count: topic.as_ref().map_or(0, |t| t.partition.max(1)),
        })
        .collect();

    let cursor = req
        .cursor
        .as_ref()
        .map(|c| (c.topic_name.to_string(), c.partition_index));
    let limit = effective_partition_limit(req);
    let (pages, next_cursor) = paginate_topic_partitions(&counts, cursor, limit);

    let topics_resp: Vec<DescribeTopicPartitionsResponseTopic> = pages
        .into_iter()
        .map(|(topic_idx, range)| {
            let (name, topic) = &topics[topic_idx];
            let name = Some(TopicName(StrBytes::from(name.clone())));
            match topic {
                None => DescribeTopicPartitionsResponseTopic::default()
                    .with_error_code(ResponseError::UnknownTopicOrPartition.code())
                    .with_name(name)
                    .with_is_internal(false)
                    .with_partitions(vec![])
                    .with_topic_authorized_operations(ALL_OPERATIONS_AUTHORIZED),
                Some(topic) => {
                    let partitions = range
                        .map(|i| describe_topic_partition(i as i32, topic, sdm))
                        .collect();
                    DescribeTopicPartitionsResponseTopic::default()
                        .with_error_code(0)
                        .with_topic_id(topic_uuid(get_tenant(), &topic.topic_name))
                        .with_name(name)
                        .with_is_internal(false)
                        .with_partitions(partitions)
                        .with_topic_authorized_operations(ALL_OPERATIONS_AUTHORIZED)
                }
            }
        })
        .collect();

    let next_cursor = next_cursor.map(|(name, partition_index)| {
        Cursor::default()
            .with_topic_name(TopicName(StrBytes::from(name)))
            .with_partition_index(partition_index)
    });

    Some(KafkaPacket::DescribeTopicPartitionsResponse(
        DescribeTopicPartitionsResponse::default()
            .with_topics(topics_resp)
            .with_next_cursor(next_cursor),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pairs: &[(&str, u32)]) -> Vec<TopicPartitionCount> {
        pairs
            .iter()
            .map(|(name, partition_count)| TopicPartitionCount {
                name: name.to_string(),
                partition_count: *partition_count,
            })
            .collect()
    }

    #[test]
    fn paginate_topic_partitions_returns_everything_when_budget_is_not_exceeded() {
        let topics = counts(&[("t1", 2), ("t2", 1)]);
        let (pages, next) = paginate_topic_partitions(&topics, None, 10);
        assert_eq!(pages, vec![(0, 0..2), (1, 0..1)]);
        assert_eq!(next, None);
    }

    #[test]
    fn paginate_topic_partitions_resumes_mid_topic_from_cursor() {
        let topics = counts(&[("t1", 3), ("t2", 1)]);
        let (pages, next) = paginate_topic_partitions(&topics, Some(("t1".to_string(), 2)), 10);
        assert_eq!(pages, vec![(0, 2..3), (1, 0..1)]);
        assert_eq!(next, None);
    }

    #[test]
    fn paginate_topic_partitions_cuts_off_mid_topic_when_budget_runs_out() {
        let topics = counts(&[("t1", 3), ("t2", 2)]);
        let (pages, next) = paginate_topic_partitions(&topics, None, 2);
        assert_eq!(pages, vec![(0, 0..2)]);
        assert_eq!(next, Some(("t1".to_string(), 2)));
    }

    #[test]
    fn paginate_topic_partitions_always_includes_unknown_topic_regardless_of_budget() {
        // "missing" topic has partition_count 0 (reported as an error entry
        // with no partitions), so it never consumes budget and is included
        // as soon as the walk reaches it, even with zero budget left.
        let topics = counts(&[("t1", 2), ("missing", 0), ("t3", 1)]);
        let (pages, next) = paginate_topic_partitions(&topics, None, 2);
        assert_eq!(pages, vec![(0, 0..2), (1, 0..0)]);
        assert_eq!(next, Some(("t3".to_string(), 0)));
    }
}
