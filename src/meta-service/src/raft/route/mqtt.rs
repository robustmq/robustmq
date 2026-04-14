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

use crate::core::cache::MetaCacheManager;
use crate::core::error::MetaServiceError;
use crate::storage::mqtt::acl::AclStorage;
use crate::storage::mqtt::blacklist::MqttBlackListStorage;
use crate::storage::mqtt::connector::MqttConnectorStorage;
use crate::storage::mqtt::group_leader::MqttGroupLeaderStorage;
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
use crate::storage::mqtt::session::MqttSessionStorage;
use crate::storage::mqtt::subscribe::MqttSubscribeStorage;
use crate::storage::mqtt::topic::MqttTopicStorage;
use crate::storage::mqtt::user::SecurityUserStorage;
use crate::storage::topic_delete::TopicDeleteStorage;
use broker_core::cache::NodeCacheManager;
use bytes::Bytes;
use common_base::tools::now_millis;
use delay_task::manager::DelayTaskManager;
use delay_task::{DelayTask, DelayTaskData};
use metadata_struct::auth::acl::SecurityAcl;
use metadata_struct::auth::blacklist::SecurityBlackList;
use metadata_struct::auth::user::SecurityUser;
use metadata_struct::connector::MQTTConnector;
use metadata_struct::mqtt::auto_subscribe::MqttAutoSubscribeRule;
use metadata_struct::mqtt::group_leader::MqttGroupLeader;
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use metadata_struct::mqtt::retain_message::MQTTRetainMessage;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::subscribe::MqttSubscribe;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use prost::Message as _;
use protocol::meta::meta_service_mqtt::{
    CreateAclRequest, CreateAutoSubscribeRuleRequest, CreateBlacklistRequest,
    CreateConnectorRequest, CreateSessionRequest, CreateTopicRequest,
    CreateTopicRewriteRuleRequest, CreateUserRequest, DeleteAclRequest,
    DeleteAutoSubscribeRuleRequest, DeleteBlacklistRequest, DeleteConnectorRequest,
    DeleteSessionRequest, DeleteSubscribeRequest, DeleteTopicRequest,
    DeleteTopicRewriteRuleRequest, DeleteUserRequest, SaveLastWillMessageRequest,
    SetSubscribeRequest, SetTopicRetainMessageRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

#[derive(Clone)]
pub struct DataRouteMqtt {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    cache_manager: Arc<MetaCacheManager>,
    broker_cache: Arc<NodeCacheManager>,
    pub delay_task_manager: Arc<DelayTaskManager>,
}
impl DataRouteMqtt {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<MetaCacheManager>,
        broker_cache: Arc<NodeCacheManager>,
        delay_task_manager: Arc<DelayTaskManager>,
    ) -> Self {
        DataRouteMqtt {
            rocksdb_engine_handler,
            cache_manager,
            broker_cache,
            delay_task_manager,
        }
    }

    // User
    pub fn create_user(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateUserRequest::decode(value.as_ref())?;
        let storage = SecurityUserStorage::new(self.rocksdb_engine_handler.clone());
        let user = SecurityUser::decode(&req.content)?;
        storage.save(&req.tenant, &req.user_name, user.clone())?;
        Ok(())
    }

    pub fn delete_user(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteUserRequest::decode(value.as_ref())?;
        let storage = SecurityUserStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.user_name)?;
        Ok(())
    }

    // Topic
    pub async fn create_topic(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateTopicRequest::decode(value.as_ref())?;
        let topic = Topic::decode(&req.content)?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(topic.clone())?;
        Ok(())
    }

    pub fn delete_topic(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteTopicRequest::decode(value.as_ref())?;
        // save topic
        let topic_storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        let mut topic = match topic_storage.get(&req.tenant, &req.topic_name)? {
            Some(t) => t,
            None => return Ok(()),
        };
        topic.mark_delete = true;
        topic_storage.save(topic.clone())?;

        // save topic delete
        let delete_storage = TopicDeleteStorage::new(self.rocksdb_engine_handler.clone());
        delete_storage.save(&topic)?;
        Ok(())
    }

    // Retain Message
    pub fn set_retain_message(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SetTopicRetainMessageRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());

        if storage.get(&req.tenant, &req.topic_name)?.is_none() {
            return Ok(());
        }

        if let Some(retain) = req.retain_message {
            let message = MQTTRetainMessage::decode(&retain)?;
            storage.save_retain_message(message)?;
        }

        Ok(())
    }

    pub fn delete_retain_message(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SetTopicRetainMessageRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());

        if storage.get(&req.tenant, &req.topic_name)?.is_none() {
            return Ok(());
        }
        storage.delete_retain_message(&req.tenant, &req.topic_name)?;
        Ok(())
    }

    // LastWill Message
    pub fn save_last_will_message(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SaveLastWillMessageRequest::decode(value.as_ref())?;
        let storage = MqttLastWillStorage::new(self.rocksdb_engine_handler.clone());
        let last_will_message = MqttLastWillData::decode(&req.last_will_message)?;
        storage.save(&req.client_id, last_will_message)?;
        Ok(())
    }

    // Session
    pub async fn create_session(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateSessionRequest::decode(value.as_ref())?;
        let storage = MqttSessionStorage::new(self.rocksdb_engine_handler.clone());

        let mut persist_sessions = Vec::new();
        for raw in &req.sessions {
            let session = MqttSession::decode(&raw.session)?;
            if session.is_persist_session {
                persist_sessions.push(session.clone());
            } else {
                self.broker_cache.add_session(session.clone());
            }

            let is_session_expire = session.connection_id.is_none()
                && session.broker_id.is_none()
                && session.distinct_time.is_some();

            // If it is a disconnected connection, it needs to be added to the queue for session expiration
            if is_session_expire {
                if let Some(distinct_time) = session.distinct_time {
                    let target_time = session.session_expiry_interval + distinct_time;
                    let task = DelayTask::build_ephemeral(
                        session.client_id.clone(),
                        DelayTaskData::MQTTSessionExpire(
                            session.tenant.clone(),
                            session.client_id.clone(),
                        ),
                        target_time,
                    );

                    self.delay_task_manager.create_task(task).await?;
                }
            } else if self.delay_task_manager.contains_task(&session.client_id) {
                self.delay_task_manager
                    .delete_task(&session.client_id)
                    .await?;
            }
        }

        if !persist_sessions.is_empty() {
            storage.save_batch(&persist_sessions)?;
        }

        Ok(())
    }

    pub fn delete_session(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteSessionRequest::decode(value.as_ref())?;
        let storage = MqttSessionStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.client_id)?;
        self.broker_cache
            .delete_session(&req.tenant, &req.client_id);
        Ok(())
    }

    // TopicRewriteRule
    pub fn create_topic_rewrite_rule(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateTopicRewriteRuleRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        let topic_rewrite_rule = MqttTopicRewriteRule {
            name: req.name.clone(),
            desc: req.desc.clone(),
            tenant: req.tenant.clone(),
            action: req.action.clone(),
            source_topic: req.source_topic.clone(),
            dest_topic: req.dest_topic.clone(),
            regex: req.regex.clone(),
            timestamp: now_millis(),
        };
        storage.save_topic_rewrite_rule(&topic_rewrite_rule)
    }

    pub fn delete_topic_rewrite_rule(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteTopicRewriteRuleRequest::decode(value.as_ref())?;
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete_topic_rewrite_rule(&req.tenant, &req.name)
    }

    // Subscribe
    pub fn set_subscribe(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let storage = MqttSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        let req = SetSubscribeRequest::decode(value.as_ref())?;
        let subscribe = MqttSubscribe::decode(&req.subscribe)?;
        storage.save(&req.client_id, &req.path, subscribe)?;
        Ok(())
    }

    pub fn delete_subscribe(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let storage = MqttSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        let req = DeleteSubscribeRequest::decode(value.as_ref())?;
        if !req.path.is_empty() {
            storage.delete_by_path(&req.client_id, &req.path)?;
        }

        if !req.client_id.is_empty() {
            storage.delete_by_client_id(&req.client_id)?;
        }

        Ok(())
    }

    // Connector
    pub fn set_connector(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let storage = MqttConnectorStorage::new(self.rocksdb_engine_handler.clone());
        let req = CreateConnectorRequest::decode(value.as_ref())?;
        let connector = MQTTConnector::decode(&req.connector)?;
        storage.save(&req.connector_name, &connector)?;
        self.cache_manager.add_connector(connector);
        Ok(())
    }

    pub fn delete_connector(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let storage = MqttConnectorStorage::new(self.rocksdb_engine_handler.clone());
        let req = DeleteConnectorRequest::decode(value.as_ref())?;
        storage.delete(&req.connector_name)?;
        self.cache_manager.remove_connector(&req.connector_name);
        Ok(())
    }

    // ACL
    pub fn create_acl(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateAclRequest::decode(value.as_ref())?;
        let acl_storage = AclStorage::new(self.rocksdb_engine_handler.clone());
        let acl = SecurityAcl::decode(&req.acl)?;
        acl_storage.save(acl)?;
        Ok(())
    }

    pub fn delete_acl(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteAclRequest::decode(value.as_ref())?;
        let acl_storage = AclStorage::new(self.rocksdb_engine_handler.clone());
        acl_storage.delete(&req.tenant, &req.name)?;
        Ok(())
    }

    // BlackList
    pub fn create_blacklist(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateBlacklistRequest::decode(value.as_ref())?;
        let blacklist_storage = MqttBlackListStorage::new(self.rocksdb_engine_handler.clone());
        let blacklist = SecurityBlackList::decode(&req.blacklist)?;
        blacklist_storage.save(blacklist)?;
        Ok(())
    }

    pub fn delete_blacklist(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteBlacklistRequest::decode(value.as_ref())?;
        let blacklist_storage = MqttBlackListStorage::new(self.rocksdb_engine_handler.clone());
        blacklist_storage.delete(&req.tenant, &req.name)?;
        Ok(())
    }

    // Group Leader
    pub fn create_group_leader(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let leader = MqttGroupLeader::decode(&value)?;
        let storage = MqttGroupLeaderStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(&leader.tenant, &leader.group_name, leader.broker_id)?;
        self.cache_manager.add_group_leader(leader);
        Ok(())
    }

    pub fn delete_group_leader(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let leader = MqttGroupLeader::decode(&value)?;
        let storage = MqttGroupLeaderStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&leader.tenant, &leader.group_name)?;
        self.cache_manager
            .remove_group_leader(&leader.tenant, &leader.group_name);
        Ok(())
    }

    // AutoSubscribeRule
    pub fn create_auto_subscribe_rule(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = CreateAutoSubscribeRuleRequest::decode(value.as_ref())?;
        let rule = MqttAutoSubscribeRule::decode(&req.content)
            .map_err(|e| MetaServiceError::CommonError(e.to_string()))?;
        let storage = MqttSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        storage.save_auto_subscribe_rule(&rule)
    }

    pub fn delete_auto_subscribe_rule(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteAutoSubscribeRuleRequest::decode(value.as_ref())?;
        let storage = MqttSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete_auto_subscribe_rule(&req.tenant, &req.name)
    }
}
