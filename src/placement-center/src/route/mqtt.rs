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

use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::topic::MqttTopic;
use prost::Message as _;
use protocol::placement_center::placement_center_mqtt::{
    CreateSessionRequest, CreateUserRequest, DeleteExclusiveTopicRequest, DeleteSessionRequest,
    DeleteTopicRequest, DeleteUserRequest, SaveLastWillMessageRequest, SetExclusiveTopicRequest,
    UpdateSessionRequest,
};

use crate::core::error::PlacementCenterError;
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
use crate::storage::mqtt::session::MqttSessionStorage;
use crate::storage::mqtt::topic::MqttTopicStorage;
use crate::storage::mqtt::user::MqttUserStorage;
use crate::storage::rocksdb::RocksDBEngine;

#[derive(Debug, Clone)]
pub struct DataRouteMqtt {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
}
impl DataRouteMqtt {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        DataRouteMqtt {
            rocksdb_engine_handler,
        }
    }

    pub fn create_user(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateUserRequest::decode(value.as_ref())?;
        let storage = MqttUserStorage::new(self.rocksdb_engine_handler.clone());
        let user = serde_json::from_slice(&req.content)?;
        storage.save(&req.cluster_name, &req.user_name, user)?;
        Ok(())
    }

    pub fn delete_user(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteUserRequest::decode(value.as_ref())?;
        let storage = MqttUserStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.cluster_name, &req.user_name)?;
        Ok(())
    }

    pub fn create_topic(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let topic = serde_json::from_slice::<MqttTopic>(&value)?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(&topic.cluster_name, &topic.topic_name, topic.clone())?;
        Ok(())
    }

    pub fn delete_topic(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteTopicRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.cluster_name, &req.topic_name)?;
        Ok(())
    }

    pub fn save_last_will_message(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = SaveLastWillMessageRequest::decode(value.as_ref())?;
        let storage = MqttLastWillStorage::new(self.rocksdb_engine_handler.clone());
        let last_will_message = serde_json::from_slice(&req.last_will_message)?;
        storage.save(&req.cluster_name, &req.client_id, last_will_message)?;
        Ok(())
    }

    pub fn create_session(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateSessionRequest::decode(value.as_ref())?;
        let storage = MqttSessionStorage::new(self.rocksdb_engine_handler.clone());
        let session = serde_json::from_slice::<MqttSession>(&req.session)?;
        storage.save(&req.cluster_name, &req.client_id, session)?;
        Ok(())
    }

    pub fn update_session(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = UpdateSessionRequest::decode(value.as_ref())?;
        let storage = MqttSessionStorage::new(self.rocksdb_engine_handler.clone());

        if let Some(mut session) = storage.get(&req.cluster_name, &req.client_id)? {
            if req.connection_id > 0 {
                session.update_connnction_id(Some(req.connection_id));
            } else {
                session.update_connnction_id(None);
            }

            if req.broker_id > 0 {
                session.update_broker_id(Some(req.broker_id));
            } else {
                session.update_broker_id(None);
            }

            if req.reconnect_time > 0 {
                session.reconnect_time = Some(req.reconnect_time);
            }

            if req.distinct_time > 0 {
                session.distinct_time = Some(req.distinct_time);
            } else {
                session.distinct_time = None;
            }

            storage.save(&req.cluster_name, &req.client_id, session)?;
        }

        Ok(())
    }

    pub fn delete_session(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteSessionRequest::decode(value.as_ref())?;
        let storage = MqttSessionStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.cluster_name, &req.client_id)?;
        Ok(())
    }

    pub fn set_nx_exclusive_topic(&self, value: Vec<u8>) -> Result<bool, PlacementCenterError> {
        let req = SetExclusiveTopicRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.set_nx_exclisve_topic(&req.cluster_name, &req.topic_name, value.clone())
    }
    pub fn delete_exclusive_topic(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteExclusiveTopicRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete_exclisve_topic(&req.cluster_name, &req.topic_name)
    }
}
