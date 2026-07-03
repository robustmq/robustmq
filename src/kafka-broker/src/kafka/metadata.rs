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
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::metadata_response::{
    MetadataResponseBroker, MetadataResponsePartition, MetadataResponseTopic,
};
use kafka_protocol::messages::{MetadataRequest, MetadataResponse, TopicName};
use kafka_protocol::protocol::StrBytes;
use metadata_struct::tenant::DEFAULT_TENANT;
use metadata_struct::topic::Topic;
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::driver::StorageDriverManager;

pub fn process_metadata(
    broker_cache: Option<&Arc<NodeCacheManager>>,
    storage_driver_manager: Option<&Arc<StorageDriverManager>>,
    req: &MetadataRequest,
) -> Option<KafkaPacket> {
    let (topics, brokers, controller_id) = match broker_cache {
        Some(cache) => (
            build_topics_from_cache(cache, storage_driver_manager, req),
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
    storage_driver_manager: Option<&Arc<StorageDriverManager>>,
    req: &MetadataRequest,
) -> Vec<MetadataResponseTopic> {
    let requested = req.topics.as_deref().unwrap_or(&[]);

    if requested.is_empty() {
        return cache
            .list_topics_by_tenant(DEFAULT_TENANT)
            .into_iter()
            .map(|topic| topic_to_metadata(topic, storage_driver_manager))
            .collect();
    }

    requested
        .iter()
        .filter_map(|t| t.name.clone())
        .map(
            |name| match cache.get_topic_by_name(DEFAULT_TENANT, &name) {
                Some(topic) => topic_to_metadata(topic, storage_driver_manager),
                None => MetadataResponseTopic::default()
                    .with_error_code(ResponseError::UnknownTopicOrPartition.code())
                    .with_name(Some(name))
                    .with_is_internal(false)
                    .with_partitions(vec![]),
            },
        )
        .collect()
}

fn topic_to_metadata(
    topic: Topic,
    storage_driver_manager: Option<&Arc<StorageDriverManager>>,
) -> MetadataResponseTopic {
    let partitions = (0..topic.partition.max(1))
        .map(|i| partition_metadata(i as i32, &topic, storage_driver_manager))
        .collect();
    MetadataResponseTopic::default()
        .with_error_code(0)
        .with_name(Some(TopicName(StrBytes::from(topic.topic_name))))
        .with_is_internal(false)
        .with_partitions(partitions)
}

// Partition leader/replicas/ISR are read from the shard's active segment
// (owned by storage-engine's ISR/leader-rebalance machinery) rather than
// tracked separately here, so failover and rebalancing stay in sync
// automatically. Falls back to broker 0 when the shard has no active
// segment yet (e.g. topic storage type without ISR, or not yet created).
fn partition_metadata(
    partition_index: i32,
    topic: &Topic,
    storage_driver_manager: Option<&Arc<StorageDriverManager>>,
) -> MetadataResponsePartition {
    let segment = storage_driver_manager.and_then(|sdm| {
        let shard_name = topic.storage_name_list.get(&(partition_index as u32))?;
        sdm.engine_storage_handler
            .cache_manager
            .get_active_segment(shard_name)
    });

    let (leader_id, replica_nodes, isr_nodes) = match segment {
        Some(segment) => (
            segment.leader as i32,
            segment
                .replicas
                .iter()
                .map(|r| r.node_id as i32)
                .collect::<Vec<_>>(),
            segment
                .isr
                .iter()
                .map(|&node_id| node_id as i32)
                .collect::<Vec<_>>(),
        ),
        None => (0, vec![0], vec![0]),
    };

    MetadataResponsePartition::default()
        .with_error_code(0)
        .with_partition_index(partition_index)
        .with_leader_id(leader_id.into())
        .with_replica_nodes(replica_nodes.into_iter().map(Into::into).collect())
        .with_isr_nodes(isr_nodes.into_iter().map(Into::into).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kafka_protocol::messages::BrokerId;
    use metadata_struct::topic::Topic;

    #[test]
    fn split_host_port_parses_valid_addr() {
        assert_eq!(
            split_host_port("127.0.0.1:9092"),
            Some(("127.0.0.1".to_string(), 9092))
        );
    }

    #[test]
    fn split_host_port_rejects_invalid_input() {
        assert_eq!(split_host_port("no-port-here"), None);
        assert_eq!(split_host_port("127.0.0.1:abc"), None);
    }

    #[test]
    fn partition_metadata_falls_back_without_storage_driver_manager() {
        let topic = Topic::new("tenant", "topic", common_config::storage::StorageType::EngineSegment);
        let partition = partition_metadata(0, &topic, None);
        assert_eq!(partition.error_code, 0);
        assert_eq!(partition.partition_index, 0);
        assert_eq!(partition.leader_id, 0);
        assert_eq!(partition.replica_nodes, vec![BrokerId(0)]);
        assert_eq!(partition.isr_nodes, vec![BrokerId(0)]);
    }
}
