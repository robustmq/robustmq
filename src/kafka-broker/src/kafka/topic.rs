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

use broker_core::topic::TopicStorage;
use common_config::{broker::broker_config, storage::StorageType};
use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::create_partitions_request::CreatePartitionsTopic;
use kafka_protocol::messages::create_partitions_response::CreatePartitionsTopicResult;
use kafka_protocol::messages::create_topics_request::CreatableTopic;
use kafka_protocol::messages::create_topics_response::CreatableTopicResult;
use kafka_protocol::messages::delete_topics_response::DeletableTopicResult;
use kafka_protocol::messages::{
    CreatePartitionsRequest, CreatePartitionsResponse, CreateTopicsRequest, CreateTopicsResponse,
    DeleteRecordsRequest, DeleteTopicsRequest, DeleteTopicsResponse, TopicName,
};
use kafka_protocol::protocol::StrBytes;
use metadata_struct::tenant::DEFAULT_TENANT;
use metadata_struct::topic::{Topic, TopicConfig, TopicSource};
use uuid::Uuid;

use crate::core::constants::{USE_DEFAULT_PARTITIONS, USE_DEFAULT_REPLICATION_FACTOR};
use protocol::kafka::packet::KafkaPacket;
use storage_adapter::{
    driver::StorageDriverManager,
    topic::{create_topic_full, update_topic_partitions_full},
};
use tracing::warn;

fn topic_error(
    name: kafka_protocol::messages::TopicName,
    err: ResponseError,
) -> CreatableTopicResult {
    CreatableTopicResult::default()
        .with_name(name)
        .with_error_code(err.code())
}

// Kafka's retention.ms is the only config with a direct RobustMQ equivalent
// (TopicConfig.retention_sec); everything else is silently ignored rather
// than rejected, since most client-sent configs have no local meaning.
fn apply_supported_configs(
    config: &mut TopicConfig,
    configs: &[kafka_protocol::messages::create_topics_request::CreatableTopicConfig],
) {
    for c in configs {
        if c.name.as_str() == "retention.ms" {
            if let Some(ms) = c
                .value
                .as_ref()
                .and_then(|v| v.as_str().parse::<u64>().ok())
            {
                config.retention_sec = ms / 1000;
            }
        }
    }
}

async fn create_one_topic(
    sdm: &Arc<StorageDriverManager>,
    creatable: &CreatableTopic,
    validate_only: bool,
) -> CreatableTopicResult {
    let topic_name = creatable.name.to_string();

    if sdm
        .broker_cache
        .get_topic_by_name(DEFAULT_TENANT, &topic_name)
        .is_some()
    {
        return topic_error(creatable.name.clone(), ResponseError::TopicAlreadyExists);
    }

    if !creatable.assignments.is_empty() {
        // Manual per-partition replica placement isn't supported: replicas are
        // always assigned automatically by the storage layer's leader-rebalance.
        return topic_error(
            creatable.name.clone(),
            ResponseError::InvalidReplicaAssignment,
        );
    }

    let conf = broker_config();
    let partition = match creatable.num_partitions {
        USE_DEFAULT_PARTITIONS => conf.runtime.default_topic_partition_num,
        n if n >= 1 => n as u32,
        _ => return topic_error(creatable.name.clone(), ResponseError::InvalidPartitions),
    };
    let replication = match creatable.replication_factor {
        USE_DEFAULT_REPLICATION_FACTOR => conf.runtime.default_topic_replica_num,
        n if n >= 1 => n as u32,
        _ => {
            return topic_error(
                creatable.name.clone(),
                ResponseError::InvalidReplicationFactor,
            );
        }
    };

    let mut config = TopicConfig::default();
    apply_supported_configs(&mut config, &creatable.configs);

    let topic = Topic::new(DEFAULT_TENANT, &topic_name, StorageType::EngineSegment)
        .with_source(TopicSource::Kafka)
        .with_partition(partition)
        .with_replication(replication)
        .with_config(config);

    if validate_only {
        return CreatableTopicResult::default()
            .with_name(creatable.name.clone())
            .with_error_code(0);
    }

    match create_topic_full(
        &sdm.broker_cache,
        sdm,
        &sdm.engine_storage_handler.client_pool,
        &topic,
    )
    .await
    {
        Ok(()) => CreatableTopicResult::default()
            .with_name(creatable.name.clone())
            .with_error_code(0)
            .with_num_partitions(partition as i32)
            .with_replication_factor(replication as i16),
        Err(e) => {
            warn!("Kafka CreateTopics failed for {}: {}", topic_name, e);
            topic_error(creatable.name.clone(), ResponseError::UnknownServerError)
        }
    }
}

pub async fn process_create_topics(
    sdm: &Arc<StorageDriverManager>,
    req: &CreateTopicsRequest,
) -> Option<KafkaPacket> {
    let mut results = Vec::with_capacity(req.topics.len());
    for creatable in &req.topics {
        results.push(create_one_topic(sdm, creatable, req.validate_only).await);
    }

    Some(KafkaPacket::CreateTopicsResponse(
        CreateTopicsResponse::default().with_topics(results),
    ))
}

fn delete_error(name: Option<TopicName>, err: ResponseError) -> DeletableTopicResult {
    DeletableTopicResult::default()
        .with_name(name)
        .with_error_code(err.code())
}

// Resolves a delete request's target to a topic_name. `name` takes precedence
// when present (the common case); `topic_id` is only consulted when no name
// was given (v6 delete-by-ID).
async fn resolve_topic_name(
    sdm: &Arc<StorageDriverManager>,
    name: Option<&TopicName>,
    topic_id: Uuid,
) -> Option<String> {
    if let Some(name) = name {
        return Some(name.to_string());
    }
    if topic_id.is_nil() {
        return None;
    }

    // TODO(perf/correctness): this is an O(topic count) scan, and today it can
    // never actually match — RobustMQ's Topic.topic_id is an xid-based string
    // (common_base::uuid::unique_id), not a real UUID, and CreateTopics/Metadata
    // never hand clients a UUID-formatted topic_id to round-trip here. Revisit
    // once topic_id has a real UUID representation end-to-end. Delete-by-ID is
    // rare enough that the scan cost is acceptable in the meantime.
    let id_str = topic_id.to_string();
    sdm.broker_cache
        .list_topics_by_tenant(DEFAULT_TENANT)
        .into_iter()
        .find(|t| t.topic_id == id_str)
        .map(|t| t.topic_name)
}

async fn delete_one_topic(
    sdm: &Arc<StorageDriverManager>,
    name: Option<&TopicName>,
    topic_id: Uuid,
) -> DeletableTopicResult {
    let Some(topic_name) = resolve_topic_name(sdm, name, topic_id).await else {
        return delete_error(name.cloned(), ResponseError::UnknownTopicOrPartition);
    };
    let response_name = Some(TopicName(StrBytes::from(topic_name.clone())));

    let Some(topic) = sdm
        .broker_cache
        .get_topic_by_name(DEFAULT_TENANT, &topic_name)
    else {
        return delete_error(response_name, ResponseError::UnknownTopicOrPartition);
    };

    if topic.source == TopicSource::SystemInner {
        return delete_error(response_name, ResponseError::TopicDeletionDisabled);
    }

    let topic_storage = TopicStorage::new(sdm.engine_storage_handler.client_pool.clone());
    match topic_storage
        .delete_topic(DEFAULT_TENANT, &topic_name)
        .await
    {
        Ok(()) => DeletableTopicResult::default()
            .with_name(response_name)
            .with_error_code(0),
        Err(e) => {
            warn!("Kafka DeleteTopics failed for {}: {}", topic_name, e);
            delete_error(response_name, ResponseError::UnknownServerError)
        }
    }
}

pub async fn process_delete_topics(
    sdm: &Arc<StorageDriverManager>,
    req: &DeleteTopicsRequest,
) -> Option<KafkaPacket> {
    let mut responses = Vec::with_capacity(req.topics.len() + req.topic_names.len());

    for t in &req.topics {
        responses.push(delete_one_topic(sdm, t.name.as_ref(), t.topic_id).await);
    }
    for name in &req.topic_names {
        responses.push(delete_one_topic(sdm, Some(name), Uuid::nil()).await);
    }

    Some(KafkaPacket::DeleteTopicsResponse(
        DeleteTopicsResponse::default().with_responses(responses),
    ))
}

pub fn process_delete_records(_req: &DeleteRecordsRequest) -> Option<KafkaPacket> {
    None
}

fn partitions_error(name: TopicName, err: ResponseError) -> CreatePartitionsTopicResult {
    CreatePartitionsTopicResult::default()
        .with_name(name)
        .with_error_code(err.code())
}

async fn create_partitions_for_topic(
    sdm: &Arc<StorageDriverManager>,
    t: &CreatePartitionsTopic,
    validate_only: bool,
) -> CreatePartitionsTopicResult {
    let topic_name = t.name.to_string();

    let Some(topic) = sdm
        .broker_cache
        .get_topic_by_name(DEFAULT_TENANT, &topic_name)
    else {
        return partitions_error(t.name.clone(), ResponseError::UnknownTopicOrPartition);
    };

    if t.assignments.as_ref().is_some_and(|a| !a.is_empty()) {
        // Manual per-partition replica placement isn't supported, same as CreateTopics.
        return partitions_error(t.name.clone(), ResponseError::InvalidReplicaAssignment);
    }

    // `count` is the new total partition count (not a delta) and must strictly increase.
    if t.count < 1 || (t.count as u32) <= topic.partition {
        return partitions_error(t.name.clone(), ResponseError::InvalidPartitions);
    }
    let new_partition = t.count as u32;

    if validate_only {
        return CreatePartitionsTopicResult::default()
            .with_name(t.name.clone())
            .with_error_code(0);
    }

    match update_topic_partitions_full(
        &sdm.broker_cache,
        sdm,
        &sdm.engine_storage_handler.client_pool,
        DEFAULT_TENANT,
        &topic_name,
        new_partition,
    )
    .await
    {
        Ok(()) => CreatePartitionsTopicResult::default()
            .with_name(t.name.clone())
            .with_error_code(0),
        Err(e) => {
            warn!("Kafka CreatePartitions failed for {}: {}", topic_name, e);
            partitions_error(t.name.clone(), ResponseError::UnknownServerError)
        }
    }
}

pub async fn process_create_partitions(
    sdm: &Arc<StorageDriverManager>,
    req: &CreatePartitionsRequest,
) -> Option<KafkaPacket> {
    let mut results = Vec::with_capacity(req.topics.len());
    for t in &req.topics {
        results.push(create_partitions_for_topic(sdm, t, req.validate_only).await);
    }

    Some(KafkaPacket::CreatePartitionsResponse(
        CreatePartitionsResponse::default().with_results(results),
    ))
}
