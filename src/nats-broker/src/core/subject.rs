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
use crate::push::{
    manager::NatsSubscribeManager,
    parse::{ParseAction, ParseSubscribeData},
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
const INNER_MQ9_SHARD_NAME: &str = "$sys.inner.mq9.shard.";

static SHARD_ALLOCATION_GENERAL: AtomicI64 = AtomicI64::new(0);

pub async fn try_get_or_init_subject(
    cache_manager: &Arc<NatsCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<NatsSubscribeManager>,
    tenant: &str,
    subject: &str,
    is_mq9: bool,
) -> Result<NatSubject, NatsBrokerError> {
    if tenant.is_empty() {
        return Err(NatsBrokerError::CommonError(
            "Tenant cannot be empty".to_string(),
        ));
    }

    if let Some(topic) = cache_manager.node_cache.get_topic_by_name(tenant, subject) {
        return Ok(topic);
    }

    let topic = if is_mq9 {
        try_get_or_init_mq9_subject(
            cache_manager,
            storage_driver_manager,
            client_pool,
            tenant,
            subject,
        )
        .await?
    } else {
        try_get_or_init_nats_core_subject(
            cache_manager,
            storage_driver_manager,
            client_pool,
            tenant,
            subject,
        )
        .await?
    };

    let data = ParseSubscribeData::new_topic(ParseAction::Add, topic.clone());
    subscribe_manager.send_parse_event(data).await;

    Ok(topic)
}

async fn try_get_or_init_mq9_subject(
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

async fn try_get_or_init_nats_core_subject(
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
