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
use tonic::{Request, Response, Status};

pub fn list_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListTopicRequest>,
) -> Result<Response<ListTopicReply>, Status> {
    let req = request.into_inner();
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    if !req.topic_name.is_empty() {
        match storage.get(&req.cluster_name, &req.topic_name) {
            Ok(Some(topic)) => {
                let topics = vec![topic.encode()];
                Ok(Response::new(ListTopicReply { topics }))
            }
            Ok(None) => Ok(Response::new(ListTopicReply { topics: vec![] })),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    } else {
        let data = match storage.list(&req.cluster_name) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        let mut result = Vec::new();
        for raw in data {
            result.push(raw.encode());
        }
        Ok(Response::new(ListTopicReply { topics: result }))
    }
}
pub async fn create_topic_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<CreateTopicRequest>,
) -> Result<Response<CreateTopicReply>, Status> {
    let req = request.into_inner();
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let topic = match topic_storage.get(&req.cluster_name, &req.topic_name) {
        Ok(topic) => topic,
        Err(e) => {
            return Err(Status::cancelled(e.to_string()));
        }
    };
    if topic.is_some() {
        return Err(Status::cancelled(
            PlacementCenterError::TopicAlreadyExist(req.topic_name).to_string(),
        ));
    };

    let data = StorageData::new(
        StorageDataType::MqttSetTopic,
        CreateTopicRequest::encode_to_vec(&req),
    );
    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };

    let topic = match serde_json::from_slice::<MqttTopic>(&req.content) {
        Ok(topic) => topic,
        Err(e) => {
            return Err(Status::cancelled(e.to_string()));
        }
    };
    if let Err(e) =
        update_cache_by_add_topic(&req.cluster_name, call_manager, client_pool, topic).await
    {
        return Err(Status::cancelled(e.to_string()));
    };

    Ok(Response::new(CreateTopicReply::default()))
}
pub async fn delete_topic_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteTopicRequest>,
) -> Result<Response<DeleteTopicReply>, Status> {
    let req = request.into_inner();
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let topic = match topic_storage.get(&req.cluster_name, &req.topic_name) {
        Ok(topic) => topic,
        Err(e) => {
            return Err(Status::cancelled(e.to_string()));
        }
    };
    if topic.is_none() {
        return Err(Status::cancelled(
            PlacementCenterError::TopicDoesNotExist(req.topic_name).to_string(),
        ));
    };

    let data = StorageData::new(
        StorageDataType::MqttDeleteTopic,
        DeleteTopicRequest::encode_to_vec(&req),
    );
    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };

    if let Err(e) =
        update_cache_by_delete_topic(&req.cluster_name, call_manager, client_pool, topic.unwrap())
            .await
    {
        return Err(Status::cancelled(e.to_string()));
    };

    Ok(Response::new(DeleteTopicReply::default()))
}

pub async fn set_topic_retain_message_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<SetTopicRetainMessageRequest>,
) -> Result<Response<SetTopicRetainMessageReply>, Status> {
    let req = request.into_inner();
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let mut topic = match topic_storage.get(&req.cluster_name, &req.topic_name) {
        Ok(Some(topic)) => topic,
        Ok(None) => {
            return Err(Status::cancelled(
                PlacementCenterError::TopicDoesNotExist(req.topic_name).to_string(),
            ));
        }
        Err(e) => {
            return Err(Status::cancelled(e.to_string()));
        }
    };
    if req.retain_message.is_empty() {
        topic.retain_message = None;
        topic.retain_message_expired_at = None;
    } else {
        topic.retain_message = Some(req.retain_message);
        topic.retain_message_expired_at = Some(req.retain_message_expired_at);
    }

    let topic_vec = match serde_json::to_vec(&topic) {
        Ok(topic_vec) => topic_vec,
        Err(e) => {
            return Err(Status::cancelled(e.to_string()));
        }
    };
    let request = CreateTopicRequest {
        cluster_name: req.cluster_name.clone(),
        topic_name: req.topic_name.clone(),
        content: topic_vec,
    };
    let data = StorageData::new(
        StorageDataType::MqttSetTopic,
        CreateTopicRequest::encode_to_vec(&request),
    );
    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };
    Ok(Response::new(SetTopicRetainMessageReply::default()))
}

pub async fn save_last_will_message_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<SaveLastWillMessageRequest>,
) -> Result<Response<SaveLastWillMessageReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttSaveLastWillMessage,
        SaveLastWillMessageRequest::encode_to_vec(&req),
    );

    match raft_machine_apply.client_write(data).await {
        Ok(_) => Ok(Response::new(SaveLastWillMessageReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn create_topic_rewrite_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<CreateTopicRewriteRuleRequest>,
) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttCreateTopicRewriteRule,
        CreateTopicRewriteRuleRequest::encode_to_vec(&req),
    );

    match raft_machine_apply.client_write(data).await {
        Ok(_) => Ok(Response::new(CreateTopicRewriteRuleReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn delete_topic_rewrite_rule_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    request: Request<DeleteTopicRewriteRuleRequest>,
) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttDeleteTopicRewriteRule,
        DeleteTopicRewriteRuleRequest::encode_to_vec(&req),
    );

    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    }

    Ok(Response::new(DeleteTopicRewriteRuleReply::default()))
}

pub fn list_topic_rewrite_rule_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListTopicRewriteRuleRequest>,
) -> Result<Response<ListTopicRewriteRuleReply>, Status> {
    let req = request.into_inner();
    let storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    match storage.list_topic_rewrite_rule(&req.cluster_name) {
        Ok(data) => {
            let mut result = Vec::new();
            for raw in data {
                result.push(raw.encode());
            }
            Ok(Response::new(ListTopicRewriteRuleReply {
                topic_rewrite_rules: result,
            }))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}
