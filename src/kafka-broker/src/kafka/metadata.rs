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

use broker_core::cache::NodeCacheManager;
use kafka_protocol::messages::metadata_response::{
    MetadataResponseBroker, MetadataResponsePartition, MetadataResponseTopic,
};
use kafka_protocol::messages::{MetadataRequest, MetadataResponse, TopicName};
use kafka_protocol::protocol::StrBytes;
use metadata_struct::tenant::DEFAULT_TENANT;
use metadata_struct::topic::Topic;
use protocol::kafka::packet::KafkaPacket;

const UNKNOWN_TOPIC_OR_PARTITION: i16 = 3;

pub fn process_metadata(
    broker_cache: Option<&Arc<NodeCacheManager>>,
    req: &MetadataRequest,
) -> Option<KafkaPacket> {
    let (topics, brokers, controller_id) = match broker_cache {
        Some(cache) => (
            build_topics_from_cache(cache, req),
            build_brokers_from_cache(cache),
            pick_controller_id(cache),
        ),
        None => (Vec::new(), Vec::new(), 0),
    };

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

fn split_host_port(addr: &str) -> Option<(String, i32)> {
    let (host, port) = addr.rsplit_once(':')?;
    let port = port.parse::<i32>().ok()?;
    Some((host.to_string(), port))
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
    req: &MetadataRequest,
) -> Vec<MetadataResponseTopic> {
    let requested = req.topics.as_deref().unwrap_or(&[]);

    if requested.is_empty() {
        return cache
            .list_topics_by_tenant(DEFAULT_TENANT)
            .into_iter()
            .map(topic_to_metadata)
            .collect();
    }

    requested
        .iter()
        .filter_map(|t| t.name.clone())
        .map(
            |name| match cache.get_topic_by_name(DEFAULT_TENANT, &name) {
                Some(topic) => topic_to_metadata(topic),
                None => MetadataResponseTopic::default()
                    .with_error_code(UNKNOWN_TOPIC_OR_PARTITION)
                    .with_name(Some(name))
                    .with_is_internal(false)
                    .with_partitions(vec![]),
            },
        )
        .collect()
}

fn topic_to_metadata(topic: Topic) -> MetadataResponseTopic {
    let partitions = (0..topic.partition.max(1))
        .map(|i| partition_metadata(i as i32))
        .collect();
    MetadataResponseTopic::default()
        .with_error_code(0)
        .with_name(Some(TopicName(StrBytes::from(topic.topic_name))))
        .with_is_internal(false)
        .with_partitions(partitions)
}

fn partition_metadata(partition_index: i32) -> MetadataResponsePartition {
    MetadataResponsePartition::default()
        .with_error_code(0)
        .with_partition_index(partition_index)
        .with_leader_id(0.into())
        .with_replica_nodes(vec![0.into()])
        .with_isr_nodes(vec![0.into()])
}
