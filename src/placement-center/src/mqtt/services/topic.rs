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

use crate::core::error::PlacementCenterError;
use crate::mqtt::controller::call_broker::{
    update_cache_by_add_topic, update_cache_by_delete_topic, MQTTInnerCallManager,
};
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::storage::mqtt::topic::MqttTopicStorage;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::MqttTopic;
use prost::Message;
use protocol::placement_center::placement_center_mqtt::{
    CreateTopicReply, CreateTopicRequest, CreateTopicRewriteRuleReply,
    CreateTopicRewriteRuleRequest, DeleteTopicReply, DeleteTopicRequest,
    DeleteTopicRewriteRuleReply, DeleteTopicRewriteRuleRequest, ListTopicReply, ListTopicRequest,
    ListTopicRewriteRuleReply, ListTopicRewriteRuleRequest, SaveLastWillMessageReply,
    SaveLastWillMessageRequest, SetTopicRetainMessageReply, SetTopicRetainMessageRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;

pub fn list_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListTopicRequest,
) -> Result<ListTopicReply, PlacementCenterError> {
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let mut topics = Vec::new();

    if !req.topic_name.is_empty() {
        if let Some(topic) = storage.get(&req.cluster_name, &req.topic_name)? {
            topics.push(topic.encode());
        }
    } else {
        let data = storage.list(&req.cluster_name)?;
        topics = data.into_iter().map(|raw| raw.encode()).collect();
    }
    Ok(ListTopicReply { topics })
}

pub async fn create_topic_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateTopicRequest,
) -> Result<CreateTopicReply, PlacementCenterError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    if (topic_storage.get(&req.cluster_name, &req.topic_name)?).is_some() {
        return Err(PlacementCenterError::TopicAlreadyExist(
            req.topic_name.clone(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::MqttSetTopic,
        CreateTopicRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;

    let topic = serde_json::from_slice::<MqttTopic>(&req.content)?;
    update_cache_by_add_topic(&req.cluster_name, call_manager, client_pool, topic).await?;

    Ok(CreateTopicReply {})
}

pub async fn delete_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteTopicRequest,
) -> Result<DeleteTopicReply, PlacementCenterError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    let topic = topic_storage
        .get(&req.cluster_name, &req.topic_name)?
        .ok_or_else(|| PlacementCenterError::TopicDoesNotExist(req.topic_name.clone()))?;

    let data = StorageData::new(
        StorageDataType::MqttDeleteTopic,
        DeleteTopicRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    update_cache_by_delete_topic(&req.cluster_name, call_manager, client_pool, topic).await?;

    Ok(DeleteTopicReply {})
}

pub async fn set_topic_retain_message_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &SetTopicRetainMessageRequest,
) -> Result<SetTopicRetainMessageReply, PlacementCenterError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());

    let mut topic = topic_storage
        .get(&req.cluster_name, &req.topic_name)?
        .ok_or_else(|| PlacementCenterError::TopicDoesNotExist(req.topic_name.clone()))?;

    // Update retain message fields
    if req.retain_message.is_empty() {
        topic.retain_message = None;
        topic.retain_message_expired_at = None;
    } else {
        topic.retain_message = Some(req.retain_message.clone());
        topic.retain_message_expired_at = Some(req.retain_message_expired_at);
    }

    let topic_vec = serde_json::to_vec(&topic)?;
    let request = CreateTopicRequest {
        cluster_name: req.cluster_name.clone(),
        topic_name: req.topic_name.clone(),
        content: topic_vec,
    };

    let data = StorageData::new(
        StorageDataType::MqttSetTopic,
        CreateTopicRequest::encode_to_vec(&request),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(SetTopicRetainMessageReply {})
}

pub async fn save_last_will_message_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &SaveLastWillMessageRequest,
) -> Result<SaveLastWillMessageReply, PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttSaveLastWillMessage,
        SaveLastWillMessageRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(SaveLastWillMessageReply {})
}

pub async fn create_topic_rewrite_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &CreateTopicRewriteRuleRequest,
) -> Result<CreateTopicRewriteRuleReply, PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttCreateTopicRewriteRule,
        CreateTopicRewriteRuleRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    Ok(CreateTopicRewriteRuleReply {})
}

pub async fn delete_topic_rewrite_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &DeleteTopicRewriteRuleRequest,
) -> Result<DeleteTopicRewriteRuleReply, PlacementCenterError> {
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
) -> Result<ListTopicRewriteRuleReply, PlacementCenterError> {
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let data = storage.list_topic_rewrite_rule(&req.cluster_name)?;

    let rules = data.into_iter().map(|raw| raw.encode()).collect();
    Ok(ListTopicRewriteRuleReply {
        topic_rewrite_rules: rules,
    })
}
