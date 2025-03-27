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

use common_base::tools::now_mills;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use metadata_struct::mqtt::topic::MqttTopic;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use metadata_struct::mqtt::user::MqttUser;
use prost::Message as _;
use protocol::mqtt::common::{qos, retain_forward_rule, Error, QoS, RetainForwardRule};
use protocol::placement_center::placement_center_mqtt::{
    CreateAclRequest, CreateBlacklistRequest, CreateConnectorRequest, CreateSessionRequest,
    CreateTopicRequest, CreateTopicRewriteRuleRequest, CreateUserRequest, DeleteAclRequest,
    DeleteAutoSubscribeRuleRequest, DeleteBlacklistRequest, DeleteConnectorRequest,
    DeleteSessionRequest, DeleteSubscribeRequest, DeleteTopicRequest,
    DeleteTopicRewriteRuleRequest, DeleteUserRequest, SaveLastWillMessageRequest,
    SetAutoSubscribeRuleRequest, SetSubscribeRequest, UpdateSessionRequest,
};

use crate::core::error::PlacementCenterError;
use crate::mqtt::cache::MqttCacheManager;
use crate::storage::mqtt::acl::AclStorage;
use crate::storage::mqtt::blacklist::MqttBlackListStorage;
use crate::storage::mqtt::connector::MqttConnectorStorage;
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
use crate::storage::mqtt::session::MqttSessionStorage;
use crate::storage::mqtt::subscribe::MqttSubscribeStorage;
use crate::storage::mqtt::topic::MqttTopicStorage;
use crate::storage::mqtt::user::MqttUserStorage;
use crate::storage::rocksdb::RocksDBEngine;

#[derive(Debug, Clone)]
pub struct DataRouteMqtt {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    mqtt_cache: Arc<MqttCacheManager>,
}
impl DataRouteMqtt {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        mqtt_cache: Arc<MqttCacheManager>,
    ) -> Self {
        DataRouteMqtt {
            rocksdb_engine_handler,
            mqtt_cache,
        }
    }

    // User
    pub fn create_user(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateUserRequest::decode(value.as_ref())?;
        let storage = MqttUserStorage::new(self.rocksdb_engine_handler.clone());
        let user = serde_json::from_slice::<MqttUser>(&req.content)?;
        storage.save(&req.cluster_name, &req.user_name, user.clone())?;
        self.mqtt_cache.add_user(&req.cluster_name, user);
        Ok(())
    }

    pub fn delete_user(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteUserRequest::decode(value.as_ref())?;
        let storage = MqttUserStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.cluster_name, &req.user_name)?;
        self.mqtt_cache
            .remove_user(&req.cluster_name, &req.user_name);
        Ok(())
    }

    // Topic
    pub fn create_topic(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateTopicRequest::decode(value.as_ref())?;
        let topic = serde_json::from_slice::<MqttTopic>(&req.content)?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(&topic.cluster_name, &topic.topic_name, topic.clone())?;
        self.mqtt_cache
            .add_topic(&topic.cluster_name, topic.clone());
        Ok(())
    }

    pub fn delete_topic(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteTopicRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.cluster_name, &req.topic_name)?;
        self.mqtt_cache
            .remove_topic(&req.cluster_name, &req.topic_name);
        Ok(())
    }

    // LastWill Message
    pub fn save_last_will_message(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = SaveLastWillMessageRequest::decode(value.as_ref())?;
        let storage = MqttLastWillStorage::new(self.rocksdb_engine_handler.clone());
        let last_will_message = serde_json::from_slice(&req.last_will_message)?;
        storage.save(&req.cluster_name, &req.client_id, last_will_message)?;
        Ok(())
    }

    // Session
    pub fn create_session(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateSessionRequest::decode(value.as_ref())?;
        let storage = MqttSessionStorage::new(self.rocksdb_engine_handler.clone());
        let session = serde_json::from_str::<MqttSession>(&req.session)?;
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

    // TopicRewriteRule
    pub fn create_topic_rewrite_rule(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateTopicRewriteRuleRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        let topic_rewrite_rule = MqttTopicRewriteRule {
            cluster: req.cluster_name.clone(),
            action: req.action.clone(),
            source_topic: req.source_topic.clone(),
            dest_topic: req.dest_topic.clone(),
            regex: req.regex.clone(),
            timestamp: now_mills(),
        };
        storage.save_topic_rewrite_rule(
            &req.cluster_name,
            &req.action,
            &req.source_topic,
            topic_rewrite_rule,
        )
    }

    pub fn delete_topic_rewrite_rule(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteTopicRewriteRuleRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete_topic_rewrite_rule(&req.cluster_name, &req.action, &req.source_topic)
    }

    // Subscribe
    pub fn set_subscribe(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let storage = MqttSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        let req = SetSubscribeRequest::decode(value.as_ref())?;
        let subscribe = serde_json::from_slice::<MqttSubscribe>(&req.subscribe)?;
        storage.save(&req.cluster_name, &req.client_id, &req.path, subscribe)?;
        Ok(())
    }

    pub fn delete_subscribe(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let storage = MqttSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        let req = DeleteSubscribeRequest::decode(value.as_ref())?;
        if !req.path.is_empty() {
            storage.delete_by_path(&req.cluster_name, &req.client_id, &req.path)?;
        } else {
            storage.delete_by_client_id(&req.cluster_name, &req.client_id)?;
        }
        Ok(())
    }

    // Connector
    pub fn set_connector(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let storage = MqttConnectorStorage::new(self.rocksdb_engine_handler.clone());
        let req = CreateConnectorRequest::decode(value.as_ref())?;
        let connector = serde_json::from_slice::<MQTTConnector>(&req.connector)?;
        storage.save(&req.cluster_name, &req.connector_name, &connector)?;
        self.mqtt_cache.add_connector(&req.cluster_name, &connector);
        Ok(())
    }

    pub fn delete_connector(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let storage = MqttConnectorStorage::new(self.rocksdb_engine_handler.clone());
        let req = DeleteConnectorRequest::decode(value.as_ref())?;
        storage.delete(&req.cluster_name, &req.connector_name)?;
        self.mqtt_cache
            .remove_connector(&req.cluster_name, &req.connector_name);
        Ok(())
    }

    // ACL
    pub fn create_acl(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateAclRequest::decode(value.as_ref())?;
        let acl_storage = AclStorage::new(self.rocksdb_engine_handler.clone());
        let acl = serde_json::from_slice::<MqttAcl>(&req.acl)?;
        acl_storage.save(&req.cluster_name, acl)?;
        Ok(())
    }

    pub fn delete_acl(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteAclRequest::decode(value.as_ref())?;
        let acl_storage = AclStorage::new(self.rocksdb_engine_handler.clone());
        let acl = serde_json::from_slice::<MqttAcl>(&req.acl)?;
        acl_storage.delete(&req.cluster_name, &acl)?;
        Ok(())
    }

    // BlackList
    pub fn create_blacklist(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = CreateBlacklistRequest::decode(value.as_ref())?;
        let blacklist_storage = MqttBlackListStorage::new(self.rocksdb_engine_handler.clone());
        let blacklist = serde_json::from_slice::<MqttAclBlackList>(&req.blacklist)?;
        blacklist_storage.save(&req.cluster_name, blacklist)?;
        Ok(())
    }

    pub fn delete_blacklist(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteBlacklistRequest::decode(value.as_ref())?;
        let blacklist_storage = MqttBlackListStorage::new(self.rocksdb_engine_handler.clone());
        blacklist_storage.delete(&req.cluster_name, &req.blacklist_type, &req.resource_name)?;
        Ok(())
    }

    // AutoSubscribeRule
    pub fn set_auto_subscribe_rule(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = SetAutoSubscribeRuleRequest::decode(value.as_ref())?;
        let storage = MqttSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        let mut _qos: Option<QoS> = None;
        if req.qos <= u8::MAX as u32 {
            _qos = qos(req.qos as u8);
        } else {
            return Err(PlacementCenterError::CommonError(
                Error::InvalidRemainingLength(req.qos as usize).to_string(),
            ));
        };

        let mut _retained_handling: Option<RetainForwardRule> = None;
        if req.retained_handling <= u8::MAX as u32 {
            _retained_handling = retain_forward_rule(req.retained_handling as u8);
        } else {
            return Err(PlacementCenterError::CommonError(
                Error::InvalidRemainingLength(req.retained_handling as usize).to_string(),
            ));
        };

        let auto_subscribe_rule: MqttAutoSubscribeRule = MqttAutoSubscribeRule {
            cluster: req.cluster_name.clone(),
            topic: req.topic.clone(),
            qos: _qos.ok_or(PlacementCenterError::CommonError(
                Error::InvalidQoS(req.qos as u8).to_string(),
            ))?,
            no_local: req.no_local,
            retain_as_published: req.retain_as_published,
            retained_handling: _retained_handling.ok_or(PlacementCenterError::CommonError(
                Error::InvalidRetainForwardRule(req.retained_handling as u8).to_string(),
            ))?,
        };
        storage.save_auto_subscribe_rule(&req.cluster_name, &req.topic, auto_subscribe_rule)
    }

    pub fn delete_auto_subscribe_rule(&self, value: Vec<u8>) -> Result<(), PlacementCenterError> {
        let req = DeleteAutoSubscribeRuleRequest::decode(value.as_ref())?;
        let storage = MqttSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete_auto_subscribe_rule(&req.cluster_name, &req.topic)
    }
}
