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

use crate::driver::StorageDriverManager;
use broker_core::{
    cache::NodeCacheManager,
    inner_topic::{
        AGENT_REPORT_INFO_TOPIC, DELAY_QUEUE_INDEX_TOPIC, DELAY_QUEUE_MESSAGE_TOPIC,
        DELAY_TASK_INDEX_TOPIC, LAST_WILL_MESSAGE_TOPIC, QOS2_INNER_TOPIC, RETAIN_MESSAGE_TOPIC,
    },
};
use common_base::error::common::CommonError;
use common_config::{broker::broker_config, storage::StorageType};
use grpc_clients::{
    meta::mqtt::call::{placement_create_topic, placement_update_topic_partitions},
    pool::ClientPool,
};
use metadata_struct::{
    mqtt::topic::{Topic, TopicSource},
    storage::shard::EngineShardConfig,
    tenant::DEFAULT_TENANT,
};
use protocol::meta::meta_service_mqtt::{CreateTopicRequest, UpdateTopicPartitionsRequest};
use std::{sync::Arc, time::Duration};
use tokio::time::{sleep, timeout};
use tracing::{debug, info};

pub fn topic_replication_num(replica_num: u32) -> u32 {
    let conf = broker_config();
    if conf.meta_addrs.len() == 1 {
        1
    } else {
        replica_num
    }
}

fn shard_config_for(topic: &Topic) -> EngineShardConfig {
    EngineShardConfig {
        replica_num: topic.replication,
        storage_type: topic.storage_type,
        max_segment_size: topic.config.max_segment_size,
        max_record_num: topic.config.max_record_num,
        retention_sec: topic.config.retention_sec,
        is_inner_topic: topic.source == TopicSource::SystemInner,
        ..Default::default()
    }
}

pub async fn create_topic_full(
    broker_cache: &Arc<NodeCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    topic: &Topic,
) -> Result<(), CommonError> {
    let conf = broker_config();
    let request = CreateTopicRequest {
        tenant: topic.tenant.clone(),
        topic_name: topic.topic_name.clone(),
        content: topic.encode()?,
    };
    placement_create_topic(client_pool, &conf.get_meta_service_addr(), request).await?;

    // wait topic create complete with timeout (5 seconds)
    let wait_result = timeout(Duration::from_secs(5), async {
        loop {
            if broker_cache
                .get_topic_by_name(&topic.tenant, &topic.topic_name)
                .is_some()
            {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await;

    if wait_result.is_err() {
        return Err(CommonError::CommonError(format!(
            "Timeout waiting for topic '{}' to be created after 5 seconds",
            topic.topic_name
        )));
    }

    // create topic message storage
    storage_driver_manager
        .create_storage_resource(&topic.tenant, &topic.topic_name, &shard_config_for(topic))
        .await?;
    Ok(())
}

/// Grows an existing topic's partition count. `partition` is the new total
/// (not a delta) and must be greater than the topic's current partition count;
/// meta-service enforces this too, but callers should validate before calling.
pub async fn update_topic_partitions_full(
    broker_cache: &Arc<NodeCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    tenant: &str,
    topic_name: &str,
    partition: u32,
) -> Result<(), CommonError> {
    let conf = broker_config();
    let request = UpdateTopicPartitionsRequest {
        tenant: tenant.to_string(),
        topic_name: topic_name.to_string(),
        partition,
    };
    placement_update_topic_partitions(client_pool, &conf.get_meta_service_addr(), request).await?;

    // wait for the broker cache to observe the new partition count (5 seconds)
    let wait_result = timeout(Duration::from_secs(5), async {
        loop {
            if broker_cache
                .get_topic_by_name(tenant, topic_name)
                .is_some_and(|t| t.partition >= partition)
            {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await;

    if wait_result.is_err() {
        return Err(CommonError::CommonError(format!(
            "Timeout waiting for topic '{}' partition update to propagate after 5 seconds",
            topic_name
        )));
    }

    // create_storage_resource is idempotent, so this only provisions the new shards.
    let topic = broker_cache
        .get_topic_by_name(tenant, topic_name)
        .ok_or_else(|| {
            CommonError::TopicNotFoundInBrokerCache(tenant.to_string(), topic_name.to_string())
        })?;
    storage_driver_manager
        .create_storage_resource(tenant, topic_name, &shard_config_for(&topic))
        .await?;
    Ok(())
}

/// Initialize all internal (built-in) topics required by the broker.
///
/// Should be called once during startup, after the meta service is ready
/// and the broker cache is populated.
pub async fn init_inner_topics(
    broker_cache: &Arc<NodeCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<(), CommonError> {
    for topic_name in [
        RETAIN_MESSAGE_TOPIC,
        LAST_WILL_MESSAGE_TOPIC,
        DELAY_TASK_INDEX_TOPIC,
        DELAY_QUEUE_MESSAGE_TOPIC,
        DELAY_QUEUE_INDEX_TOPIC,
        AGENT_REPORT_INFO_TOPIC,
        QOS2_INNER_TOPIC,
    ] {
        init_single_inner_topic(
            broker_cache,
            storage_driver_manager,
            client_pool,
            topic_name,
        )
        .await?;
    }
    // wait cache sync
    sleep(Duration::from_secs(3)).await;
    Ok(())
}

async fn init_single_inner_topic(
    broker_cache: &Arc<NodeCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    topic_name: &str,
) -> Result<(), CommonError> {
    if let Some(topic) = broker_cache.get_topic_by_name(DEFAULT_TENANT, topic_name) {
        debug!(
            "Inner topic '{}' already exists, ensuring storage shard is provisioned",
            topic_name
        );
        storage_driver_manager
            .create_storage_resource(DEFAULT_TENANT, topic_name, &shard_config_for(&topic))
            .await?;
        return Ok(());
    }

    let conf = broker_config();
    let topic = Topic::new(DEFAULT_TENANT, topic_name, StorageType::EngineRocksDB)
        .with_partition(conf.runtime.default_topic_partition_num)
        .with_replication(topic_replication_num(
            conf.runtime.default_topic_replica_num,
        ));
    create_topic_full(broker_cache, storage_driver_manager, client_pool, &topic).await?;

    info!("Inner topic '{}' created successfully", topic_name);
    Ok(())
}
