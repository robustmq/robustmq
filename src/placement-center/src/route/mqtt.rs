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
use common_base::error::mqtt_broker::MqttBrokerError;
use metadata_struct::mqtt::session::MqttSession;
use prost::Message as _;
use protocol::placement_center::placement_center_mqtt::{
    CreateSessionRequest, CreateTopicRequest, CreateUserRequest, DeleteSessionRequest,
    DeleteTopicRequest, DeleteUserRequest, SaveLastWillMessageRequest,
    SetTopicRetainMessageRequest, UpdateSessionRequest,
};

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

    pub fn create_user(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = CreateUserRequest::decode(value.as_ref())?;
        let storage = MqttUserStorage::new(self.rocksdb_engine_handler.clone());
        let user = serde_json::from_slice(&req.content)?;
        storage.save(&req.cluster_name, &req.user_name, user)
    }

    pub fn delete_user(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = DeleteUserRequest::decode(value.as_ref())?;
        let storage = MqttUserStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.cluster_name, &req.user_name)
    }

    pub fn create_topic(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = CreateTopicRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        let topic = serde_json::from_slice(&req.content)?;
        storage.save(&req.cluster_name, &req.topic_name, topic)
    }

    pub fn delete_topic(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = DeleteTopicRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.cluster_name, &req.topic_name)
    }

    pub fn set_topic_retain_message(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req: SetTopicRetainMessageRequest =
            SetTopicRetainMessageRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.set_topic_retain_message(
            &req.cluster_name,
            &req.topic_name,
            req.retain_message,
            req.retain_message_expired_at,
        )
    }

    pub fn save_last_will_message(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = SaveLastWillMessageRequest::decode(value.as_ref())?;
        let storage = MqttLastWillStorage::new(self.rocksdb_engine_handler.clone());
        let last_will_message = serde_json::from_slice(&req.last_will_message)?;
        storage.save(&req.cluster_name, &req.client_id, last_will_message)
    }

    pub fn create_session(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = CreateSessionRequest::decode(value.as_ref())?;
        let storage = MqttSessionStorage::new(self.rocksdb_engine_handler.clone());
        let session = serde_json::from_slice::<MqttSession>(&req.session)?;
        storage.save(&req.cluster_name, &req.client_id, session)
    }

    pub fn update_session(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = UpdateSessionRequest::decode(value.as_ref())?;
        let storage = MqttSessionStorage::new(self.rocksdb_engine_handler.clone());
        let result = storage.get(&req.cluster_name, &req.client_id)?;
        if result.is_none() {
            return Err(MqttBrokerError::SessionDoesNotExist.into());
        }

        let mut session = result.unwrap();

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

        storage.save(&req.cluster_name, &req.client_id, session)
    }

    pub fn delete_session(&self, value: Vec<u8>) -> Result<(), CommonError> {
        let req = DeleteSessionRequest::decode(value.as_ref())?;
        let storage = MqttSessionStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.cluster_name, &req.client_id)
    }
}
