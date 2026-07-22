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

use common_base::error::common::CommonError;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use common_config::storage::StorageType;
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::tenant::DEFAULT_TENANT;
use metadata_struct::topic::{Topic, TopicConfig, TopicSource};
use serde::{Deserialize, Serialize};
use storage_adapter::driver::StorageDriverManager;
use storage_adapter::topic::create_topic_full;
use tracing::warn;

const UNACKED_INDEX_TOPIC: &str = "$amqp-unacked-index";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct AmqpUnackedIndexEntry {
    pub tenant: String,
    pub queue: String,
    pub offset: u64,
    pub connection_id: u64,
    pub channel_id: u16,
    pub broker_id: u64,
    pub timestamp: u64,
}

async fn declare_index_topic(sdm: &Arc<StorageDriverManager>) -> Option<Topic> {
    if let Some(topic) = sdm
        .broker_cache
        .get_topic_by_name(DEFAULT_TENANT, UNACKED_INDEX_TOPIC)
    {
        return Some(topic);
    }

    let topic = Topic::new(
        DEFAULT_TENANT,
        UNACKED_INDEX_TOPIC,
        StorageType::EngineRocksDB,
    )
    .with_source(TopicSource::AMQP)
    .with_partition(1)
    .with_replication(topic_replication_num_for_index())
    .with_config(TopicConfig::default());

    match create_topic_full(
        &sdm.broker_cache,
        sdm,
        &sdm.engine_storage_handler.client_pool,
        &topic,
    )
    .await
    {
        Ok(()) => sdm
            .broker_cache
            .get_topic_by_name(DEFAULT_TENANT, UNACKED_INDEX_TOPIC),
        Err(e) => {
            warn!("AMQP unacked index: failed to declare topic: {}", e);
            None
        }
    }
}

fn topic_replication_num_for_index() -> u32 {
    let conf = broker_config();
    storage_adapter::topic::topic_replication_num(conf.runtime.default_topic_replica_num)
}

pub(crate) async fn write_entry(
    sdm: &Arc<StorageDriverManager>,
    tenant: &str,
    queue: &str,
    offset: u64,
    connection_id: u64,
    channel_id: u16,
    broker_id: u64,
) -> Result<u64, CommonError> {
    if declare_index_topic(sdm).await.is_none() {
        return Err(CommonError::CommonError(
            "AMQP unacked index topic is not available".to_string(),
        ));
    }

    let entry = AmqpUnackedIndexEntry {
        tenant: tenant.to_string(),
        queue: queue.to_string(),
        offset,
        connection_id,
        channel_id,
        broker_id,
        timestamp: now_second(),
    };
    let body = serde_json::to_vec(&entry)
        .map_err(|e| CommonError::CommonError(format!("encode unacked index entry: {e}")))?;
    let record = AdapterWriteRecord::new(UNACKED_INDEX_TOPIC.to_string(), body);
    let resp = sdm
        .write(
            DEFAULT_TENANT,
            UNACKED_INDEX_TOPIC,
            std::slice::from_ref(&record),
            1,
        )
        .await?;
    resp.first()
        .map(|row| row.offset)
        .ok_or_else(|| CommonError::CommonError("unacked index write returned no offset".into()))
}

pub(crate) async fn delete_entry(
    sdm: &Arc<StorageDriverManager>,
    index_offset: u64,
) -> Result<(), CommonError> {
    sdm.delete_by_offsets(DEFAULT_TENANT, UNACKED_INDEX_TOPIC, &[index_offset])
        .await
}

pub(crate) async fn scan_all(
    sdm: &Arc<StorageDriverManager>,
) -> Result<Vec<(u64, AmqpUnackedIndexEntry)>, CommonError> {
    let Some(topic) = sdm
        .broker_cache
        .get_topic_by_name(DEFAULT_TENANT, UNACKED_INDEX_TOPIC)
    else {
        return Ok(Vec::new());
    };
    let Some(shard_name) = topic.storage_name_list.get(&0) else {
        return Ok(Vec::new());
    };

    let mut read_config = AdapterReadConfig::new();
    read_config.max_record_num = 10_000;
    let mut offsets = std::collections::HashMap::new();
    offsets.insert(shard_name.clone(), 0u64);

    let records = sdm
        .read_by_offset(DEFAULT_TENANT, UNACKED_INDEX_TOPIC, &offsets, &read_config)
        .await?;

    let mut results = Vec::with_capacity(records.len());
    for record in records {
        match serde_json::from_slice::<AmqpUnackedIndexEntry>(&record.data) {
            Ok(entry) => results.push((record.metadata.offset, entry)),
            Err(e) => warn!("AMQP unacked index: skipping malformed entry: {}", e),
        }
    }
    Ok(results)
}
