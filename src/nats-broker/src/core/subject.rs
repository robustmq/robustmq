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

use crate::core::{cache::NatsCacheManager, error::NatsBrokerError};
use crate::subscribe::{
    parse::{ParseAction, ParseSubscribeData},
    NatsSubscribeManager,
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

pub type NatSubject = Topic;
const INNER_NATS_CORE_SHARD_NAME: &str = "$sys.inner.nats.core.shard.";
static SHARD_ALLOCATION_GENERAL: AtomicI64 = AtomicI64::new(0);

/// Returns the Topic that backs the given NATS subject, creating it on first use.
///
/// Design: NATS subjects are not stored in their own physical shards. Instead, each
/// subject topic's `storage_name_list` points to one of the pre-allocated internal
/// core shards (`$sys.inner.nats.core.shard.*`). This keeps the number of physical
/// shards bounded regardless of how many subjects exist.
///
/// Flow:
/// 1. Cache hit → return immediately.
/// 2. Cache miss → pick a core shard via round-robin (`get_subject_storage_name`),
///    create the subject topic with that shard's storage mapping, persist it, return.
pub async fn try_get_or_init_subject(
    cache_manager: &Arc<NatsCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<NatsSubscribeManager>,
    tenant: &str,
    subject: &str,
) -> Result<NatSubject, NatsBrokerError> {
    if tenant.is_empty() {
        return Err(NatsBrokerError::CommonError(
            "Tenant cannot be empty".to_string(),
        ));
    }

    if let Some(topic) = cache_manager.node_cache.get_topic_by_name(tenant, subject) {
        return Ok(topic);
    }

    let topic = Topic::new(tenant, subject, StorageType::EngineMemory)
        .with_source(TopicSource::NATS)
        .with_storage_name_list(
            get_subject_storage_name(cache_manager, storage_driver_manager, client_pool).await?,
        );

    create_topic_full(
        &cache_manager.node_cache,
        storage_driver_manager,
        client_pool,
        &topic,
    )
    .await?;

    // Notify parse thread so existing subscriptions can be matched against this new topic.
    let data = ParseSubscribeData::new_topic(ParseAction::Add, topic.clone());
    subscribe_manager.send_parse_event(data).await;

    Ok(topic)
}

/// Picks a core shard via round-robin and returns its `storage_name_list`.
///
/// `SHARD_ALLOCATION_GENERAL` is a global atomic counter. Each call increments it
/// and takes the result mod `core_shard_num`, producing an evenly distributed shard
/// index across all callers without any locking.
///
/// If the core shard topic is already in cache (normal path after startup), the
/// `storage_name_list` is returned immediately. Otherwise the shard topic is created
/// as a fallback (e.g. if `init_nats_core_shard` was not called or `core_shard_num`
/// was increased after startup).
async fn get_subject_storage_name(
    cache_manager: &Arc<NatsCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<HashMap<u32, String>, NatsBrokerError> {
    let seq = SHARD_ALLOCATION_GENERAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        % (broker_config().nats_runtime.core_shard_num as i64);

    let shard_topic_name = format!("{}.{}", INNER_NATS_CORE_SHARD_NAME, seq);
    let tenant = DEFAULT_TENANT;

    if let Some(topic) = cache_manager
        .node_cache
        .get_topic_by_name(tenant, &shard_topic_name)
    {
        return Ok(topic.storage_name_list.clone());
    }

    let topic = Topic::new(tenant, &shard_topic_name, StorageType::EngineMemory)
        .with_source(TopicSource::NATS);

    create_topic_full(
        &cache_manager.node_cache,
        storage_driver_manager,
        client_pool,
        &topic,
    )
    .await?;

    Ok(topic.storage_name_list.clone())
}
