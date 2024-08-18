// Copyright 2023 RobustMQ Team
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

use crate::storage::{
    mqtt::{
        lastwill::MQTTLastWillStorage, session::MQTTSessionStorage, topic::MQTTTopicStorage,
        user::MQTTUserStorage,
    },
    rocksdb::RocksDBEngine,
};
use common_base::errors::RobustMQError;
use metadata_struct::mqtt::session::MQTTSession;
use prost::Message as _;
use protocol::placement_center::generate::mqtt::{
    CreateSessionRequest, CreateTopicRequest, CreateUserRequest, DeleteSessionRequest,
    DeleteTopicRequest, DeleteUserRequest, SaveLastWillMessageRequest,
    SetTopicRetainMessageRequest, UpdateSessionRequest,
};
use std::sync::Arc;

pub struct DataRouteMQTT {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
}
impl DataRouteMQTT {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        return DataRouteMQTT {
            rocksdb_engine_handler,
        };
    }

    pub fn create_user(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = CreateUserRequest::decode(value.as_ref())?;
        let storage = MQTTUserStorage::new(self.rocksdb_engine_handler.clone());
        let user = serde_json::from_slice(&req.content)?;
        return storage.save(&req.cluster_name, &req.user_name, user);
    }

    pub fn delete_user(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = DeleteUserRequest::decode(value.as_ref())?;
        let storage = MQTTUserStorage::new(self.rocksdb_engine_handler.clone());
        return storage.delete(&req.cluster_name, &req.user_name);
    }

    pub fn create_topic(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = CreateTopicRequest::decode(value.as_ref())?;
        let storage = MQTTTopicStorage::new(self.rocksdb_engine_handler.clone());
        let topic = serde_json::from_slice(&req.content)?;
        return storage.save(&req.cluster_name, &req.topic_name, topic);
    }

    pub fn delete_topic(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = DeleteTopicRequest::decode(value.as_ref())?;
        let storage = MQTTTopicStorage::new(self.rocksdb_engine_handler.clone());
        return storage.delete(&req.cluster_name, &req.topic_name);
    }

    pub fn set_topic_retain_message(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: SetTopicRetainMessageRequest =
            SetTopicRetainMessageRequest::decode(value.as_ref())?;
        let storage = MQTTTopicStorage::new(self.rocksdb_engine_handler.clone());
        return storage.set_topic_retain_message(
            &req.cluster_name,
            &req.topic_name,
            req.retain_message,
            req.retain_message_expired_at,
        );
    }

    pub fn save_last_will_message(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = SaveLastWillMessageRequest::decode(value.as_ref())?;
        let storage = MQTTLastWillStorage::new(self.rocksdb_engine_handler.clone());
        let last_will_message = serde_json::from_slice(&req.last_will_message)?;
        return storage.save(&req.cluster_name, &req.client_id, last_will_message);
    }

    pub fn create_session(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = CreateSessionRequest::decode(value.as_ref())?;
        let storage = MQTTSessionStorage::new(self.rocksdb_engine_handler.clone());
        let session = serde_json::from_slice::<MQTTSession>(&req.session)?;
        return storage.save(&req.cluster_name, &req.client_id, session);
    }

    pub fn update_session(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = UpdateSessionRequest::decode(value.as_ref())?;
        let storage = MQTTSessionStorage::new(self.rocksdb_engine_handler.clone());
        let result = storage.get(&req.cluster_name, &req.client_id)?;
        if result.is_none() {
            return Err(RobustMQError::SessionDoesNotExist);
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

        return storage.save(&req.cluster_name, &req.client_id, session);
    }

    pub fn delete_session(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = DeleteSessionRequest::decode(value.as_ref())?;
        let storage = MQTTSessionStorage::new(self.rocksdb_engine_handler.clone());
        return storage.delete(&req.cluster_name, &req.client_id);
    }
}
