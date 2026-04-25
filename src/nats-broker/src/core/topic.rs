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

// # Topic Model for NATS and MQ9
//
// Each NATS subject and each MQ9 mail ID maps to exactly one logical Topic
// (partition = 1). This is intentional: NATS subjects and MQ9 mails are
// lightweight channels that do not need partitioning.
//
// Naming convention:
//   - NATS: topic name = NATS subject name
//     e.g. subject "orders.created"  →  topic name "orders.created"
//   - MQ9:  topic name = mail ID
//     e.g. mail address "user-inbox-42"   →  topic name "user-inbox-42"

use crate::{
    core::{cache::NatsCacheManager, error::NatsBrokerError},
    push::{
        parse::{ParseAction, ParseSubscribeData},
        NatsSubscribeManager,
    },
};
use common_config::broker::broker_config;
use common_config::storage::StorageType;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    tenant::DEFAULT_TENANT,
    topic::{Topic, TopicSource},
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicI64, Arc},
};
use storage_adapter::{driver::StorageDriverManager, topic::create_topic_full};

static SHARD_ALLOCATION_GENERAL: AtomicI64 = AtomicI64::new(0);

pub type NatSubject = Topic;
const INNER_NATS_CORE_SHARD_NAME: &str = "$sys.inner.nats.core.shard.";
const INNER_MQ9_SHARD_NAME: &str = "$sys.inner.mq9.shard.";

pub async fn create_topic_by_nats_and_mq9(
    subscribe_manager: Arc<NatsSubscribeManager>,
    topic: &Topic,
) {
    subscribe_manager
        .send_parse_event(ParseSubscribeData::new_topic(
            ParseAction::Add,
            topic.clone(),
        ))
        .await;
}

pub async fn delete_topic_by_nats_and_mq9(
    subscribe_manager: Arc<NatsSubscribeManager>,
    topic: &Topic,
) {
    subscribe_manager
        .send_parse_event(ParseSubscribeData::new_topic(
            ParseAction::Remove,
            topic.clone(),
        ))
        .await;
}

pub async fn try_get_or_init_mq9_subject(
    cache_manager: &Arc<NatsCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    tenant: &str,
    subject: &str,
) -> Result<NatSubject, NatsBrokerError> {
    let storage_name_list =
        get_subject_storage_name(cache_manager, storage_driver_manager, client_pool, true).await?;

    let topic = Topic::new(tenant, subject, StorageType::EngineRocksDB)
        .with_source(TopicSource::MQ9)
        .with_storage_name_list(storage_name_list);

    create_topic_full(
        &cache_manager.node_cache,
        storage_driver_manager,
        client_pool,
        &topic,
    )
    .await?;

    Ok(topic)
}

pub async fn try_get_or_init_nats_core_subject(
    cache_manager: &Arc<NatsCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    tenant: &str,
    subject: &str,
) -> Result<NatSubject, NatsBrokerError> {
    let storage_name_list =
        get_subject_storage_name(cache_manager, storage_driver_manager, client_pool, false).await?;

    let topic = Topic::new(tenant, subject, StorageType::EngineMemory)
        .with_source(TopicSource::NATS)
        .with_storage_name_list(storage_name_list);

    create_topic_full(
        &cache_manager.node_cache,
        storage_driver_manager,
        client_pool,
        &topic,
    )
    .await?;

    Ok(topic)
}

// Returns the shared inner shard name list for a given subject/mail topic.
//
// NATS subjects and MQ9 mails do NOT get their own dedicated storage shard.
// Instead they share a small, pre-allocated pool of inner shards:
//
//   NATS  →  $sys.inner.nats.core.shard.<seq>  (EngineMemory)
//   MQ9   →  $sys.inner.mq9.shard.<seq>        (EngineRocksDB)
//
// The shard is chosen round-robin via `SHARD_ALLOCATION_GENERAL`. Messages
// from different subjects/mails land in the same underlying shard and are
// distinguished at read time by tag (= topic name) via `read_by_tag`.
//
// Rationale: subjects and mails are numerous and ephemeral — each carries few
// messages that expire quickly. One shard per subject/mail would create an
// unbounded number of shards in the storage engine. Sharing a fixed pool
// avoids that cost while keeping messages logically isolated through tags.
async fn get_subject_storage_name(
    cache_manager: &Arc<NatsCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    is_mq9: bool,
) -> Result<HashMap<u32, String>, NatsBrokerError> {
    let seq = SHARD_ALLOCATION_GENERAL
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        .unsigned_abs()
        % (broker_config().nats_runtime.core_shard_num as u64);

    let shard_topic_name = if is_mq9 {
        format!("{}.{}", INNER_MQ9_SHARD_NAME, seq)
    } else {
        format!("{}.{}", INNER_NATS_CORE_SHARD_NAME, seq)
    };

    let tenant = DEFAULT_TENANT;

    if let Some(topic) = cache_manager
        .node_cache
        .get_topic_by_name(tenant, &shard_topic_name)
    {
        return Ok(topic.storage_name_list.clone());
    }

    let topic = if is_mq9 {
        Topic::new(tenant, &shard_topic_name, StorageType::EngineRocksDB)
            .with_source(TopicSource::MQ9)
    } else {
        Topic::new(tenant, &shard_topic_name, StorageType::EngineMemory)
            .with_source(TopicSource::NATS)
    };

    create_topic_full(
        &cache_manager.node_cache,
        storage_driver_manager,
        client_pool,
        &topic,
    )
    .await?;

    Ok(topic.storage_name_list.clone())
}
