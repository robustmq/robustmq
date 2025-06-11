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

use crate::common::pkid_manager::PkidManager;
use crate::observability::system_topic::sysmon::SystemAlarmEventMessage;
use crate::security::acl::metadata::AclMetadata;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use metadata_struct::mqtt::cluster::MqttClusterDynamicConfig;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::topic::MqttTopic;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use metadata_struct::mqtt::user::MqttUser;
use protocol::mqtt::common::{MqttProtocol, PublishProperties};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

#[derive(Clone, Serialize, Deserialize)]
pub enum MetadataCacheAction {
    Set,
    Del,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MetadataCacheType {
    Cluster,
    User,
    Topic,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MetadataChangeData {
    pub action: MetadataCacheAction,
    pub data_type: MetadataCacheType,
    pub value: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionLiveTime {
    pub protocol: MqttProtocol,
    pub keep_live: u16,
    pub heartbeat: u64,
}

#[derive(Clone)]
pub struct QosAckPacketInfo {
    pub sx: Sender<QosAckPackageData>,
    pub create_time: u128,
}

#[derive(Clone, Debug)]
pub struct QosAckPackageData {
    pub ack_type: QosAckPackageType,
    pub pkid: u16,
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum QosAckPackageType {
    PubAck,
    PubComp,
    PubRel,
    PubRec,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientPkidData {
    pub client_id: String,
    pub create_time: u64,
}

#[derive(Clone)]
pub struct CacheManager {
    pub client_pool: Arc<ClientPool>,

    // cluster_name
    pub cluster_name: String,

    // (cluster_name, Cluster)
    pub cluster_info: DashMap<String, MqttClusterDynamicConfig>,

    // (username, User)
    pub user_info: DashMap<String, MqttUser>,

    // (client_id, Session)
    pub session_info: DashMap<String, MqttSession>,

    // (connect_id, Connection)
    pub connection_info: DashMap<u64, MQTTConnection>,

    // (topic_name, Topic)
    pub topic_info: DashMap<String, MqttTopic>,

    // (topic_id, topic_name)
    pub topic_id_name: DashMap<String, String>,

    // (client_id, HeartbeatShard)
    pub heartbeat_data: DashMap<String, ConnectionLiveTime>,

    // acl metadata
    pub acl_metadata: AclMetadata,

    // pkid manager
    pub pkid_meatadata: PkidManager,

    // All topic rewrite rule
    pub topic_rewrite_rule: DashMap<String, MqttTopicRewriteRule>,

    // All auto subscribe rule
    pub auto_subscribe_rule: DashMap<String, MqttAutoSubscribeRule>,

    // Alarm Info
    pub alarm_events: DashMap<String, SystemAlarmEventMessage>,
}

impl CacheManager {
    pub fn new(client_pool: Arc<ClientPool>, cluster_name: String) -> Self {
        CacheManager {
            client_pool,
            cluster_name,
            cluster_info: DashMap::with_capacity(1),
            user_info: DashMap::with_capacity(8),
            session_info: DashMap::with_capacity(8),
            topic_info: DashMap::with_capacity(8),
            topic_id_name: DashMap::with_capacity(8),
            connection_info: DashMap::with_capacity(8),
            heartbeat_data: DashMap::with_capacity(8),
            acl_metadata: AclMetadata::new(),
            pkid_meatadata: PkidManager::new(),
            topic_rewrite_rule: DashMap::with_capacity(8),
            auto_subscribe_rule: DashMap::with_capacity(8),
            alarm_events: DashMap::with_capacity(8),
        }
    }

    // session
    pub fn add_session(&self, client_id: &str, session: &MqttSession) {
        self.session_info
            .insert(client_id.to_owned(), session.to_owned());
    }

    pub fn get_session_info(&self, client_id: &str) -> Option<MqttSession> {
        if let Some(session) = self.session_info.get(client_id) {
            return Some(session.clone());
        }
        None
    }

    pub fn update_session_connect_id(&self, client_id: &str, connect_id: Option<u64>) {
        if let Some(mut session) = self.session_info.get_mut(client_id) {
            session.update_connnction_id(connect_id);
            if connect_id.is_none() {
                session.update_distinct_time()
            }
        }
    }

    pub fn remove_session(&self, client_id: &str) {
        self.session_info.remove(client_id);
        self.heartbeat_data.remove(client_id);
        self.pkid_meatadata.remove_by_client_id(client_id);
    }

    // user
    pub fn add_user(&self, user: MqttUser) {
        self.user_info.insert(user.username.clone(), user);
    }

    pub fn del_user(&self, username: String) {
        self.user_info.remove(&username);
    }

    pub fn retain_users(&self, usernames: HashSet<String>) {
        self.user_info
            .retain(|username, _| usernames.contains(username));
    }

    // connection
    pub fn add_connection(&self, connect_id: u64, conn: MQTTConnection) {
        if let Some(mut session) = self.session_info.get_mut(&conn.client_id) {
            session.connection_id = Some(connect_id);
            self.connection_info.insert(connect_id, conn);
        }
    }

    pub fn remove_connection(&self, connect_id: u64) {
        self.connection_info.remove(&connect_id);
    }

    pub fn get_connect_id(&self, client_id: &str) -> Option<u64> {
        if let Some(sess) = self.session_info.get(client_id) {
            if let Some(conn_id) = sess.connection_id {
                return Some(conn_id);
            }
        }
        None
    }

    pub fn get_connection(&self, connect_id: u64) -> Option<MQTTConnection> {
        if let Some(conn) = self.connection_info.get(&connect_id) {
            return Some(conn.clone());
        }
        None
    }

    // create a function get the number of connections from connection_info
    pub fn get_connection_count(&self) -> usize {
        self.connection_info.len()
    }

    // topic
    pub fn add_topic(&self, topic_name: &str, topic: &MqttTopic) {
        self.topic_info.insert(topic_name.to_owned(), topic.clone());
        self.topic_id_name
            .insert(topic.topic_id.clone(), topic_name.to_owned());
    }

    pub fn delete_topic(&self, topic_name: &String, topic: &MqttTopic) {
        self.topic_info.remove(topic_name);
        self.topic_id_name.remove(&topic.topic_id);
    }

    pub fn topic_exists(&self, topic: &str) -> bool {
        self.topic_info.contains_key(topic)
    }

    pub fn topic_name_by_id(&self, topic_id: &str) -> Option<String> {
        if let Some(data) = self.topic_id_name.get(topic_id) {
            return Some(data.clone());
        }
        None
    }

    pub fn get_topic_by_name(&self, topic_name: &str) -> Option<MqttTopic> {
        if let Some(topic) = self.topic_info.get(topic_name) {
            return Some(topic.clone());
        }
        None
    }

    pub fn update_topic_retain_message(&self, topic_name: &str, retain_message: Option<Vec<u8>>) {
        if let Some(mut topic) = self.topic_info.get_mut(topic_name) {
            topic.retain_message = retain_message;
        }
    }

    // topic rewrite rule
    pub fn add_topic_rewrite_rule(&self, topic_rewrite_rule: MqttTopicRewriteRule) {
        let key = self.topic_rewrite_rule_key(
            &self.cluster_name,
            &topic_rewrite_rule.action,
            &topic_rewrite_rule.source_topic,
        );
        self.topic_rewrite_rule.insert(key, topic_rewrite_rule);
    }

    pub fn delete_topic_rewrite_rule(&self, cluster: &str, action: &str, source_topic: &str) {
        let key = self.topic_rewrite_rule_key(cluster, action, source_topic);
        self.topic_rewrite_rule.remove(&key);
    }

    pub fn get_all_topic_rewrite_rule(&self) -> Vec<MqttTopicRewriteRule> {
        self.topic_rewrite_rule
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn login_success(&self, connect_id: u64, user_name: String) {
        if let Some(mut conn) = self.connection_info.get_mut(&connect_id) {
            conn.login_success(user_name)
        }
    }

    pub fn is_login(&self, connect_id: u64) -> bool {
        if let Some(conn) = self.connection_info.get(&connect_id) {
            return conn.is_login;
        }
        false
    }

    // topic alias
    pub fn get_topic_alias(&self, connect_id: u64, topic_alias: u16) -> Option<String> {
        if let Some(conn) = self.connection_info.get(&connect_id) {
            if let Some(topic_name) = conn.topic_alias.get(&topic_alias) {
                return Some(topic_name.clone());
            } else {
                return None;
            }
        }
        None
    }

    pub fn topic_alias_exists(&self, connect_id: u64, topic_alias: u16) -> bool {
        if let Some(conn) = self.connection_info.get(&connect_id) {
            return conn.topic_alias.contains_key(&topic_alias);
        }
        false
    }

    pub fn add_topic_alias(
        &self,
        connect_id: u64,
        topic_name: &str,
        publish_properties: &Option<PublishProperties>,
    ) {
        if let Some(properties) = publish_properties {
            if let Some(alias) = properties.topic_alias {
                if let Some(conn) = self.connection_info.get_mut(&connect_id) {
                    conn.topic_alias.insert(alias, topic_name.to_owned());
                }
            }
        }
    }

    // heartbeat
    pub fn report_heartbeat(&self, client_id: String, live_time: ConnectionLiveTime) {
        self.heartbeat_data.insert(client_id, live_time);
    }

    pub fn remove_heartbeat(&self, client_id: &str) {
        self.heartbeat_data.remove(client_id);
    }

    // acl
    pub fn add_acl(&self, acl: MqttAcl) {
        self.acl_metadata.parse_mqtt_acl(acl);
    }

    pub fn remove_acl(&self, acl: MqttAcl) {
        self.acl_metadata.remove_mqtt_acl(acl);
    }

    pub fn retain_acls(&self, user_acl: HashSet<String>, client_acl: HashSet<String>) {
        self.acl_metadata
            .acl_user
            .retain(|username, _| user_acl.contains(username));
        self.acl_metadata
            .acl_client_id
            .retain(|client_id, _| client_acl.contains(client_id));
    }

    // blacklist
    pub fn add_blacklist(&self, blacklist: MqttAclBlackList) {
        self.acl_metadata.parse_mqtt_blacklist(blacklist);
    }

    pub fn remove_blacklist(&self, blacklist: MqttAclBlackList) {
        self.acl_metadata.remove_mqtt_blacklist(blacklist);
    }

    // key
    pub fn topic_rewrite_rule_key(
        &self,
        cluster: &str,
        action: &str,
        source_topic: &str,
    ) -> String {
        format!("{}_{}_{}", cluster, action, source_topic)
    }

    // auto subscribe rule
    pub fn auto_subscribe_rule_key(&self, cluster: &str, topic: &str) -> String {
        format!("{}_{}", cluster, topic)
    }

    pub fn add_auto_subscribe_rule(&self, auto_subscribe_rule: MqttAutoSubscribeRule) {
        let key = self.auto_subscribe_rule_key(&self.cluster_name, &auto_subscribe_rule.topic);
        self.auto_subscribe_rule.insert(key, auto_subscribe_rule);
    }

    pub fn delete_auto_subscribe_rule(&self, cluster: &str, topic: &str) {
        let key = self.auto_subscribe_rule_key(cluster, topic);
        self.auto_subscribe_rule.remove(&key);
    }

    pub fn add_alarm_event(&self, alarm_name: String, event: SystemAlarmEventMessage) {
        self.alarm_events.insert(alarm_name, event);
    }

    pub fn get_alarm_event(&self, name: &str) -> Option<SystemAlarmEventMessage> {
        if let Some(event) = self.alarm_events.get(name) {
            return Some(event.clone());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_get_a_alarm_event_is_empty() {
        let client_pool = Arc::new(ClientPool::new(1));
        let cache_manager = CacheManager::new(client_pool, "test_cluster".to_string());

        let event = cache_manager.get_alarm_event("test_event");
        assert!(event.is_none());
    }

    #[tokio::test]
    async fn test_add_and_get_alarm_event() {
        let client_pool = Arc::new(ClientPool::new(1));
        let cache_manager = CacheManager::new(client_pool, "test_cluster".to_string());

        let event = SystemAlarmEventMessage {
            name: "test_event".to_string(),
            message: "This is a test event".to_string(),
            activate_at: chrono::Utc::now().timestamp(),
            activated: true,
        };

        cache_manager.add_alarm_event("test_event".to_string(), event.clone());
        let retrieved_event = cache_manager.get_alarm_event("test_event");

        assert!(retrieved_event.is_some());
        assert_eq!(event.name, retrieved_event.unwrap().name);
    }
}
