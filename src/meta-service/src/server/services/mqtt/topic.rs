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

use crate::controller::mqtt::call_broker::{
    update_cache_by_add_topic, update_cache_by_delete_topic, MQTTInnerCallManager,
};
use crate::core::error::MetaServiceError;
use crate::raft::route::apply::RaftMachineManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
use crate::storage::mqtt::topic::MqttTopicStorage;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::MQTTTopic;
use prost::Message;
use protocol::meta::meta_service_mqtt::{
    CreateTopicReply, CreateTopicRequest, CreateTopicRewriteRuleReply,
    CreateTopicRewriteRuleRequest, DeleteTopicReply, DeleteTopicRequest,
    DeleteTopicRewriteRuleReply, DeleteTopicRewriteRuleRequest, GetLastWillMessageReply,
    GetLastWillMessageRequest, GetTopicRetainMessageReply, GetTopicRetainMessageRequest,
    ListTopicReply, ListTopicRequest, ListTopicRewriteRuleReply, ListTopicRewriteRuleRequest,
    SaveLastWillMessageReply, SaveLastWillMessageRequest, SetTopicRetainMessageReply,
    SetTopicRetainMessageRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

pub async fn list_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListTopicRequest,
) -> Result<Pin<Box<dyn Stream<Item = Result<ListTopicReply, Status>> + Send>>, MetaServiceError> {
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let mut topics = Vec::new();

    if !req.topic_name.is_empty() {
        if let Some(topic) = storage.get(&req.cluster_name, &req.topic_name)? {
            topics.push(topic.encode()?);
        }
    } else {
        let data = storage.list(&req.cluster_name)?;
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
    raft_machine_apply: &Arc<RaftMachineManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateTopicRequest,
) -> Result<CreateTopicReply, MetaServiceError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    if topic_storage
        .get(&req.cluster_name, &req.topic_name)?
        .is_some()
    {
        return Err(MetaServiceError::TopicAlreadyExist(req.topic_name.clone()));
    }

    let data = StorageData::new(
        StorageDataType::MqttSetTopic,
        CreateTopicRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;

    let topic = MQTTTopic::decode(&req.content)?;
    update_cache_by_add_topic(&req.cluster_name, call_manager, client_pool, topic).await?;

    Ok(CreateTopicReply {})
}

pub async fn delete_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteTopicRequest,
) -> Result<DeleteTopicReply, MetaServiceError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    let topic = topic_storage
        .get(&req.cluster_name, &req.topic_name)?
        .ok_or_else(|| MetaServiceError::TopicDoesNotExist(req.topic_name.clone()))?;

    let data = StorageData::new(
        StorageDataType::MqttDeleteTopic,
        DeleteTopicRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    update_cache_by_delete_topic(&req.cluster_name, call_manager, client_pool, topic).await?;

    Ok(DeleteTopicReply {})
}

pub async fn set_topic_retain_message_by_req(
    raft_machine_apply: &Arc<RaftMachineManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &SetTopicRetainMessageRequest,
) -> Result<SetTopicRetainMessageReply, MetaServiceError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    topic_storage
        .get(&req.cluster_name, &req.topic_name)?
        .ok_or_else(|| MetaServiceError::TopicDoesNotExist(req.topic_name.clone()))?;

    // Update retain message fields
    if req.retain_message.is_none() {
        let data = StorageData::new(
            StorageDataType::MqttDeleteRetainMessage,
            SetTopicRetainMessageRequest::encode_to_vec(req),
        );
        raft_machine_apply.client_write(data).await?;
        return Ok(SetTopicRetainMessageReply {});
    }

    let data = StorageData::new(
        StorageDataType::MqttSetRetainMessage,
        SetTopicRetainMessageRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(SetTopicRetainMessageReply {})
}

pub async fn get_topic_retain_message_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetTopicRetainMessageRequest,
) -> Result<GetTopicRetainMessageReply, MetaServiceError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    if let Some(message) = topic_storage.get_retain_message(&req.cluster_name, &req.topic_name)? {
        return Ok(GetTopicRetainMessageReply {
            retain_message: Some(message.retain_message.to_vec()),
            retain_message_expired_at: message.retain_message_expired_at,
        });
    }

    Ok(GetTopicRetainMessageReply {
        retain_message: None,
        retain_message_expired_at: 0,
    })
}

pub async fn save_last_will_message_by_req(
    raft_machine_apply: &Arc<RaftMachineManager>,
    req: &SaveLastWillMessageRequest,
) -> Result<SaveLastWillMessageReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttSaveLastWillMessage,
        SaveLastWillMessageRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(SaveLastWillMessageReply {})
}

pub async fn get_last_will_message_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetLastWillMessageRequest,
) -> Result<GetLastWillMessageReply, MetaServiceError> {
    let storage = MqttLastWillStorage::new(rocksdb_engine_handler.clone());
    if let Some(will) = storage.get(&req.cluster_name, &req.client_id)? {
        return Ok(GetLastWillMessageReply {
            message: will.encode()?,
        });
    }

    Ok(GetLastWillMessageReply {
        message: Vec::new(),
    })
}

pub async fn create_topic_rewrite_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineManager>,
    req: &CreateTopicRewriteRuleRequest,
) -> Result<CreateTopicRewriteRuleReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttCreateTopicRewriteRule,
        CreateTopicRewriteRuleRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(CreateTopicRewriteRuleReply {})
}

pub async fn delete_topic_rewrite_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineManager>,
    req: &DeleteTopicRewriteRuleRequest,
) -> Result<DeleteTopicRewriteRuleReply, MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::MqttDeleteTopicRewriteRule,
        DeleteTopicRewriteRuleRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(DeleteTopicRewriteRuleReply {})
}

pub fn list_topic_rewrite_rule_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListTopicRewriteRuleRequest,
) -> Result<ListTopicRewriteRuleReply, MetaServiceError> {
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let data = storage.list_topic_rewrite_rule(&req.cluster_name)?;

    let rules = data
        .into_iter()
        .map(|raw| raw.encode())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ListTopicRewriteRuleReply {
        topic_rewrite_rules: rules,
    })
}
