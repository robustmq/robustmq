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

use crate::core::error::MetaServiceError;
use crate::core::notify::{
    send_notify_by_add_topic, send_notify_by_create_topic_rewrite_rule,
    send_notify_by_delete_topic, send_notify_by_delete_topic_rewrite_rule,
};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::mqtt::topic::MqttTopicStorage;
use common_base::tools::now_millis;
use common_base::utils::serialize::encode_to_bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use node_call::NodeCallManager;
use protocol::meta::meta_service_mqtt::{
    CreateTopicReply, CreateTopicRequest, CreateTopicRewriteRuleReply,
    CreateTopicRewriteRuleRequest, DeleteTopicReply, DeleteTopicRequest,
    DeleteTopicRewriteRuleReply, DeleteTopicRewriteRuleRequest, GetTopicRetainMessageReply,
    GetTopicRetainMessageRequest, ListTopicReply, ListTopicRequest, ListTopicRewriteRuleReply,
    ListTopicRewriteRuleRequest, SetTopicRetainMessageReply, SetTopicRetainMessageRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

// Topic Operations
pub async fn list_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListTopicRequest,
) -> Result<Pin<Box<dyn Stream<Item = Result<ListTopicReply, Status>> + Send>>, MetaServiceError> {
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let mut topics = Vec::new();

    if !req.topic_name.is_empty() {
        let tenant = if req.tenant.is_empty() {
            ""
        } else {
            &req.tenant
        };
        if let Some(topic) = storage.get(tenant, &req.topic_name)? {
            topics.push(topic.encode()?);
        }
    } else if !req.tenant.is_empty() {
        let data = storage.list_by_tenant(&req.tenant)?;
        topics = data
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
    } else {
        let data = storage.list()?;
        topics = data
            .into_iter()
            .map(|raw| raw.encode())
            .collect::<Result<Vec<_>, _>>()?;
    }

    let output = async_stream::try_stream! {
        for topic in topics {
            yield ListTopicReply {
                topic,
            };
        }
    };

    Ok(Box::pin(output))
}

pub async fn create_topic_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateTopicRequest,
) -> Result<CreateTopicReply, MetaServiceError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    // interface maintains the idempotent semantics.
    if topic_storage.get(&req.tenant, &req.topic_name)?.is_some() {
        return Ok(CreateTopicReply {});
    }

    let data = StorageData::new(StorageDataType::MqttSetTopic, encode_to_bytes(req));
    raft_manager.write_data(&req.topic_name, data).await?;

    let topic = Topic::decode(&req.content)?;

    send_notify_by_add_topic(call_manager, topic).await?;
    Ok(CreateTopicReply {})
}

pub async fn delete_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    _client_pool: &Arc<ClientPool>,
    req: &DeleteTopicRequest,
) -> Result<DeleteTopicReply, MetaServiceError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    // Get topic to delete (must exist)
    let topic = topic_storage
        .get(&req.tenant, &req.topic_name)?
        .ok_or_else(|| MetaServiceError::TopicDoesNotExist(req.topic_name.clone()))?;

    let data = StorageData::new(StorageDataType::MqttDeleteTopic, encode_to_bytes(req));
    raft_manager.write_data(&req.topic_name, data).await?;

    send_notify_by_delete_topic(call_manager, topic).await?;

    Ok(DeleteTopicReply {})
}

// Retain Message Operations
pub async fn set_topic_retain_message_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &SetTopicRetainMessageRequest,
) -> Result<SetTopicRetainMessageReply, MetaServiceError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    // Verify topic exists
    topic_storage
        .get(&req.tenant, &req.topic_name)?
        .ok_or_else(|| MetaServiceError::TopicDoesNotExist(req.topic_name.clone()))?;

    let (data_type, data) = if req.retain_message.is_none() {
        (
            StorageDataType::MqttDeleteRetainMessage,
            encode_to_bytes(req),
        )
    } else {
        (StorageDataType::MqttSetRetainMessage, encode_to_bytes(req))
    };

    let data = StorageData::new(data_type, data);
    raft_manager.write_data(&req.topic_name, data).await?;

    Ok(SetTopicRetainMessageReply {})
}

pub async fn get_topic_retain_message_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetTopicRetainMessageRequest,
) -> Result<GetTopicRetainMessageReply, MetaServiceError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    let (retain_message, retain_message_expired_at) =
        match topic_storage.get_retain_message(&req.tenant, &req.topic_name)? {
            Some(message) => (
                Some(message.retain_message.to_vec()),
                message.retain_message_expired_at,
            ),
            None => (None, 0),
        };

    Ok(GetTopicRetainMessageReply {
        retain_message,
        retain_message_expired_at,
    })
}

// Topic Rewrite Rule Operations
pub async fn create_topic_rewrite_rule_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    call_manager: &Arc<NodeCallManager>,
    req: &CreateTopicRewriteRuleRequest,
) -> Result<CreateTopicRewriteRuleReply, MetaServiceError> {
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    if storage
        .get_topic_rewrite_rule(&req.tenant, &req.name)?
        .is_some()
    {
        return Err(MetaServiceError::CommonError(format!(
            "Topic rewrite rule '{}' for tenant '{}' already exists",
            req.name, req.tenant
        )));
    }

    let data = StorageData::new(
        StorageDataType::MqttCreateTopicRewriteRule,
        encode_to_bytes(req),
    );
    raft_manager.write_metadata(data).await?;

    let rule = MqttTopicRewriteRule {
        name: req.name.clone(),
        desc: req.desc.clone(),
        tenant: req.tenant.clone(),
        action: req.action.clone(),
        source_topic: req.source_topic.clone(),
        dest_topic: req.dest_topic.clone(),
        regex: req.regex.clone(),
        timestamp: now_millis(),
    };
    send_notify_by_create_topic_rewrite_rule(call_manager, rule).await?;

    Ok(CreateTopicRewriteRuleReply {})
}

pub async fn delete_topic_rewrite_rule_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    call_manager: &Arc<NodeCallManager>,
    req: &DeleteTopicRewriteRuleRequest,
) -> Result<DeleteTopicRewriteRuleReply, MetaServiceError> {
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let rule = storage
        .get_topic_rewrite_rule(&req.tenant, &req.name)?
        .ok_or_else(|| {
            MetaServiceError::CommonError(format!(
                "Topic rewrite rule '{}' for tenant '{}' does not exist",
                req.name, req.tenant
            ))
        })?;

    let data = StorageData::new(
        StorageDataType::MqttDeleteTopicRewriteRule,
        encode_to_bytes(req),
    );
    raft_manager.write_metadata(data).await?;

    send_notify_by_delete_topic_rewrite_rule(call_manager, rule).await?;

    Ok(DeleteTopicRewriteRuleReply {})
}

pub fn list_topic_rewrite_rule_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListTopicRewriteRuleRequest,
) -> Result<ListTopicRewriteRuleReply, MetaServiceError> {
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let data = if req.tenant.is_empty() {
        storage.list_all_topic_rewrite_rules()?
    } else {
        storage.list_topic_rewrite_rules_by_tenant(&req.tenant)?
    };

    let rules = data
        .into_iter()
        .map(|raw| raw.encode())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ListTopicRewriteRuleReply {
        topic_rewrite_rules: rules,
    })
}
