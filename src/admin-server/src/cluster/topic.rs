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

// Topic rewrite rules (list/create/delete) have been moved to:
// src/admin-server/src/mqtt/topic_rewrite.rs

use crate::{
    state::HttpState,
    tool::extractor::ValidatedJson,
    tool::{
        query::{apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::extract::{Query, State};
use broker_core::topic::TopicStorage;
use common_base::error::common::CommonError;
use common_base::http_response::{error_response, success_response};
use common_config::storage::StorageType;
use metadata_struct::adapter::adapter_shard::AdapterShardDetail;
use metadata_struct::mqtt::{retain_message::MQTTRetainMessage, topic::Topic};
use metadata_struct::topic::{Topic as MetaTopic, TopicSource};
use mqtt_broker::subscribe::manager::TopicSubscribeInfo;
use mqtt_broker::{core::error::MqttBrokerError, storage::retain::RetainStorage};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};
use storage_adapter::topic::create_topic_full;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TopicListReq {
    pub tenant: Option<String>,
    pub topic_name: Option<String>,
    pub topic_type: Option<String>, // "all", "normal", "system"
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicDetailReq {
    pub tenant: String,
    pub topic_name: String,
}

/// Request body for creating a topic.
///
/// Required: `tenant`, `topic_name`, `storage_type`, `source`.
/// Optional: `partition` and `replication` default to 1.
/// `storage_name_list` is auto-generated from `topic_id` and `partition`.
#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct TopicCreateReq {
    #[validate(length(min = 1, max = 128, message = "Tenant length must be between 1-128"))]
    pub tenant: String,

    #[validate(length(
        min = 1,
        max = 256,
        message = "Topic name length must be between 1-256"
    ))]
    pub topic_name: String,

    /// Storage backend. One of: EngineMemory, EngineRocksDB, EngineSegment.
    #[validate(length(min = 1, max = 64, message = "storage_type must not be empty"))]
    pub storage_type: String,

    /// Protocol source. One of: MQTT, NATS, MQ9, Kafka, AMQP, SystemInner.
    #[validate(length(min = 1, max = 32, message = "source must not be empty"))]
    pub source: String,

    /// Number of partitions. Defaults to 1.
    pub partition: Option<u32>,

    /// Replication factor. Defaults to 1.
    pub replication: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct TopicDeleteRep {
    pub tenant: String,
    #[validate(length(
        min = 1,
        max = 256,
        message = "Topic name length must be between 1-256"
    ))]
    pub topic_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicDetailResp {
    pub topic_info: Topic,
    pub retain_message: Option<MQTTRetainMessage>,
    pub sub_list: HashSet<TopicSubscribeInfo>,
    pub storage_list: HashMap<u32, AdapterShardDetail>,
}

pub async fn topic_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<TopicListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        None,
        None,
        None,
    );

    let broker_cache = &state.mqtt_context.cache_manager.node_cache;
    let topics = collect_topics(
        broker_cache,
        params.tenant.as_deref(),
        params.topic_name.as_deref(),
        params.topic_type.as_deref(),
    );

    let sorted = apply_sorting(topics, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

/// Collect topics from cache, optionally filtered by tenant, topic_name (fuzzy), and topic_type.
/// topic_type: "system" (contains '$'), "normal" (no '$'), or "all" (default).
fn collect_topics(
    broker_cache: &broker_core::cache::NodeCacheManager,
    tenant: Option<&str>,
    topic_name: Option<&str>,
    topic_type: Option<&str>,
) -> Vec<Topic> {
    let raw: Vec<Topic> = if let Some(t) = tenant {
        broker_cache.list_topics_by_tenant(t)
    } else {
        broker_cache
            .topic_list
            .iter()
            .map(|e| e.value().clone())
            .collect()
    };

    raw.into_iter()
        .filter(|t| {
            if let Some(keyword) = topic_name {
                if !t.topic_name.contains(keyword) {
                    return false;
                }
            }
            match topic_type.unwrap_or("all") {
                "system" => t.topic_name.contains('$'),
                "normal" => !t.topic_name.contains('$'),
                _ => true,
            }
        })
        .collect()
}

impl Queryable for Topic {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "topic_name" => Some(self.topic_name.clone()),
            "tenant" => Some(self.tenant.clone()),
            _ => None,
        }
    }
}

pub async fn topic_detail(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<TopicDetailReq>,
) -> String {
    let result = match read_topic_detail(&state, &params).await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    success_response(result)
}

async fn read_topic_detail(
    state: &Arc<HttpState>,
    params: &TopicDetailReq,
) -> Result<TopicDetailResp, MqttBrokerError> {
    let topic = if let Some(topic) = state
        .mqtt_context
        .cache_manager
        .node_cache
        .get_topic_by_name(&params.tenant, &params.topic_name)
    {
        topic
    } else {
        return Err(MqttBrokerError::TopicDoesNotExist(
            params.topic_name.clone(),
        ));
    };

    let storage_list = state
        .mqtt_context
        .storage_driver_manager
        .list_storage_resource(&topic.tenant, &topic.topic_name)
        .await?;

    let sub_list = state
        .mqtt_context
        .subscribe_manager
        .topic_subscribes
        .get(&topic.tenant)
        .and_then(|t| t.get(&topic.topic_name).map(|v| v.clone()))
        .unwrap_or_default();
    let storage = RetainStorage::new(state.mqtt_context.storage_driver_manager.clone());
    let retain_message = storage
        .get_retain_message(&topic.tenant, &topic.topic_name)
        .await?;

    Ok(TopicDetailResp {
        topic_info: topic,
        retain_message,
        sub_list,
        storage_list,
    })
}

pub async fn topic_create(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<TopicCreateReq>,
) -> String {
    let storage_type = match StorageType::from_str(&params.storage_type) {
        Ok(t) => t,
        Err(_) => {
            return error_response(format!(
                "Invalid storage_type '{}', must be one of: EngineMemory, EngineRocksDB, EngineSegment",
                params.storage_type
            ));
        }
    };

    let source = match params.source.as_str() {
        "MQTT" => TopicSource::MQTT,
        "NATS" => TopicSource::NATS,
        "MQ9" => TopicSource::MQ9,
        "Kafka" => TopicSource::Kafka,
        "AMQP" => TopicSource::AMQP,
        "SystemInner" => TopicSource::SystemInner,
        _ => {
            return error_response(format!(
                "Invalid source '{}', must be one of: MQTT, NATS, MQ9, Kafka, AMQP, SystemInner",
                params.source
            ));
        }
    };

    let partition = params.partition.unwrap_or(1).max(1);
    let replication = params.replication.unwrap_or(1).max(1);

    let topic = MetaTopic::new(&params.tenant, &params.topic_name, storage_type)
        .with_source(source)
        .with_partition(partition)
        .with_replication(replication);

    if let Err(e) = create_topic_full(
        &state.broker_cache,
        &state.storage_driver_manager,
        &state.client_pool,
        &topic,
    )
    .await
    .map_err(|e: CommonError| e.to_string())
    {
        return error_response(e);
    }

    success_response(topic)
}

pub async fn topic_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<TopicDeleteRep>,
) -> String {
    let topic_storage = TopicStorage::new(state.client_pool.clone());
    if let Err(e) = topic_storage
        .delete_topic(&params.tenant, &params.topic_name)
        .await
    {
        return error_response(e.to_string());
    }
    success_response("success")
}
