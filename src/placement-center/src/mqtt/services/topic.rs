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
    CreateTopicRequest, DeleteTopicRequest, ListTopicRequest, SetTopicRetainMessageRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;

pub async fn list_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: ListTopicRequest,
) -> Result<Vec<Vec<u8>>, PlacementCenterError> {
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    if !req.topic_name.is_empty() {
        if let Some(data) = storage.get(&req.cluster_name, &req.topic_name)? {
            return Ok(vec![data.encode()]);
        }
    } else {
        let data = storage.list(&req.cluster_name)?;
        let mut result = Vec::new();
        for raw in data {
            result.push(raw.encode());
        }
        return Ok(result);
    }
    Ok(Vec::new())
}

pub async fn create_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: CreateTopicRequest,
) -> Result<(), PlacementCenterError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let topic = topic_storage.get(&req.cluster_name, &req.topic_name)?;
    if topic.is_some() {
        return Err(PlacementCenterError::TopicAlreadyExist(req.topic_name));
    };

    let data = StorageData::new(
        StorageDataType::MqttSetTopic,
        CreateTopicRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    let topic = serde_json::from_slice::<MqttTopic>(&req.content)?;
    update_cache_by_add_topic(&req.cluster_name, call_manager, client_pool, topic).await?;
    Ok(())
}

pub async fn delete_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: DeleteTopicRequest,
) -> Result<(), PlacementCenterError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let topic = topic_storage.get(&req.cluster_name, &req.topic_name)?;
    if topic.is_none() {
        return Err(PlacementCenterError::TopicDoesNotExist(req.topic_name));
    };

    let data = StorageData::new(
        StorageDataType::MqttDeleteTopic,
        DeleteTopicRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    update_cache_by_delete_topic(&req.cluster_name, call_manager, client_pool, topic.unwrap())
        .await?;

    Ok(())
}

pub async fn set_topic_retain_message_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: SetTopicRetainMessageRequest,
) -> Result<(), PlacementCenterError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let mut topic = if let Some(tp) = topic_storage.get(&req.cluster_name, &req.topic_name)? {
        tp
    } else {
        return Err(PlacementCenterError::TopicDoesNotExist(req.topic_name));
    };

    if req.retain_message.is_empty() {
        topic.retain_message = None;
        topic.retain_message_expired_at = None;
    } else {
        topic.retain_message = Some(req.retain_message);
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
    Ok(())
}
