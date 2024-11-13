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

use protocol::placement_center::placement_center_mqtt::{
    CreateTopicRequest, SetTopicRetainMessageRequest,
};
use rocksdb_engine::RocksDBEngine;

use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::storage::mqtt::topic::MqttTopicStorage;

pub async fn create_topic_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    _: &Arc<PlacementCacheManager>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: CreateTopicRequest,
) -> Result<(), PlacementCenterError> {
    let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let topic = topic_storage.get(&req.cluster_name, &req.topic_name)?;
    if topic.is_some() {
        return Err(PlacementCenterError::TopicAlreadyExist(req.topic_name));
    };

    // if !cluster_cache.cluster_list.contains_key(&req.cluster_name) {
    //     return Err(PlacementCenterError::ClusterDoesNotExist(req.cluster_name));
    // }

    let data = StorageData::new(StorageDataType::MqttSetTopic, req.content);
    raft_machine_apply.client_write(data).await?;
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
    let data = StorageData::new(StorageDataType::MqttSetTopic, topic_vec);
    raft_machine_apply.client_write(data).await?;
    Ok(())
}
