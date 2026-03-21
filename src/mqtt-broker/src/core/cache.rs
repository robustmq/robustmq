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

use crate::core::pkid_manager::PkidManager;
use crate::security::auth::metadata::AclMetadata;
use broker_core::cache::NodeCacheManager;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::auth::authn_config::AuthnConfig;
use metadata_struct::mqtt::auto_subscribe::MqttAutoSubscribeRule;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::session::MqttSession;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use metadata_struct::mqtt::user::MqttUser;
use protocol::mqtt::common::{MqttProtocol, PublishProperties};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConnectionLiveTime {
    pub protocol: MqttProtocol,
    pub keep_live: u16,
    pub heartbeat: u64,
}

#[derive(Clone)]
pub struct QosAckPacketInfo {
    pub sx: mpsc::Sender<QosAckPackageData>,
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
pub struct MQTTCacheManager {
    // broker cache
    pub node_cache: Arc<NodeCacheManager>,

    // client pool
    pub client_pool: Arc<ClientPool>,

    // (tenant, (username, User))
    pub user_info: DashMap<String, DashMap<String, MqttUser>>,

    // (tenant, (client_id, Session))
    pub session_info: DashMap<String, DashMap<String, MqttSession>>,

    // (tenant, (connect_id, Connection))
    pub connection_info: DashMap<String, DashMap<u64, MQTTConnection>>,

    pub authn_list: DashMap<String, AuthnConfig>,

    // (client_id, HeartbeatShard)
    pub heartbeat_data: DashMap<String, ConnectionLiveTime>,

    // acl metadata
    pub acl_metadata: AclMetadata,

    // pkid manager
    pub pkid_manager: PkidManager,

    // (tenant, (action_source_topic, rule))
    pub topic_rewrite_rule: DashMap<String, DashMap<String, MqttTopicRewriteRule>>,

    // Topic rewrite new name: outer key = tenant, inner key = original topic_name, value = rewritten topic_name
    pub topic_rewrite_new_name: DashMap<String, DashMap<String, String>>,
    pub re_calc_topic_rewrite: Arc<RwLock<bool>>,

    // All auto subscribe rule: outer key = tenant, inner key = topic
    pub auto_subscribe_rule: DashMap<String, DashMap<String, MqttAutoSubscribeRule>>,

    // Topic is Validator
    pub topic_is_validator: DashMap<String, bool>,
}

impl MQTTCacheManager {
    pub fn new(client_pool: Arc<ClientPool>, broker_cache: Arc<NodeCacheManager>) -> Self {
        MQTTCacheManager {
            client_pool,
            node_cache: broker_cache,
            user_info: DashMap::with_capacity(8),
            session_info: DashMap::with_capacity(8),
            connection_info: DashMap::with_capacity(8),
            heartbeat_data: DashMap::with_capacity(8),
            acl_metadata: AclMetadata::new(),
            pkid_manager: PkidManager::new(),
            topic_rewrite_rule: DashMap::with_capacity(8),
            auto_subscribe_rule: DashMap::with_capacity(8),
            topic_is_validator: DashMap::with_capacity(8),
            re_calc_topic_rewrite: Arc::new(RwLock::new(false)),
            topic_rewrite_new_name: DashMap::with_capacity(8),
            authn_list: DashMap::with_capacity(2),
        }
    }

    // session
    pub fn get_session_client_id_list(&self) -> Vec<String> {
        self.session_info
            .iter()
            .flat_map(|tenant_entry| {
                tenant_entry
                    .value()
                    .iter()
                    .map(|session| session.client_id.clone())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn add_session(&self, client_id: &str, session: &MqttSession) {
        self.session_info
            .entry(session.tenant.clone())
            .or_default()
            .insert(client_id.to_owned(), session.to_owned());
    }

    pub fn get_session_info(&self, client_id: &str) -> Option<MqttSession> {
        for tenant_entry in self.session_info.iter() {
            if let Some(session) = tenant_entry.value().get(client_id) {
                return Some(session.clone());
            }
        }
        None
    }

    pub fn get_session_info_by_tenant(&self, tenant: &str, client_id: &str) -> Option<MqttSession> {
        self.session_info
            .get(tenant)
            .and_then(|m| m.get(client_id).map(|s| s.clone()))
    }

    pub fn update_session_connect_id(&self, client_id: &str, connect_id: Option<u64>) {
        for tenant_entry in self.session_info.iter() {
            if let Some(mut session) = tenant_entry.value().get_mut(client_id) {
                session.update_connection_id(connect_id);
                if connect_id.is_none() {
                    session.update_distinct_time()
                }
                return;
            }
        }
    }

    pub fn remove_session(&self, client_id: &str) {
        for tenant_entry in self.session_info.iter() {
            tenant_entry.value().remove(client_id);
        }
        self.heartbeat_data.remove(client_id);
        self.pkid_manager.remove_by_client_id(client_id);
    }

    // user
    pub fn add_user(&self, user: MqttUser) {
        self.user_info
            .entry(user.tenant.clone())
            .or_insert_with(DashMap::new)
            .insert(user.username.clone(), user);
    }

    pub fn del_user(&self, tenant: &str, username: &str) {
        if let Some(tenant_map) = self.user_info.get(tenant) {
            tenant_map.remove(username);
        }
    }

    // connection
    pub fn add_connection(&self, connect_id: u64, conn: MQTTConnection) {
        for tenant_entry in self.session_info.iter() {
            if let Some(mut session) = tenant_entry.value().get_mut(&conn.client_id) {
                session.connection_id = Some(connect_id);
                self.connection_info
                    .entry(conn.tenant.clone())
                    .or_default()
                    .insert(connect_id, conn);
                return;
            }
        }
    }

    pub fn remove_connection(&self, connect_id: u64) {
        for tenant_entry in self.connection_info.iter() {
            tenant_entry.value().remove(&connect_id);
        }
    }

    pub fn get_connect_id(&self, client_id: &str) -> Option<u64> {
        if let Some(sess) = self.get_session_info(client_id) {
            return sess.connection_id;
        }
        None
    }

    pub fn get_connection(&self, connect_id: u64) -> Option<MQTTConnection> {
        for tenant_entry in self.connection_info.iter() {
            if let Some(conn) = tenant_entry.value().get(&connect_id) {
                return Some(conn.clone());
            }
        }
        None
    }

    pub fn session_count(&self) -> usize {
        self.session_info.iter().map(|e| e.value().len()).sum()
    }

    pub fn session_count_by_tenant(&self, tenant: &str) -> usize {
        self.session_info
            .get(tenant)
            .map(|e| e.value().len())
            .unwrap_or(0)
    }

    pub fn get_connection_count(&self) -> usize {
        self.connection_info.iter().map(|e| e.value().len()).sum()
    }

    pub fn get_connection_count_by_tenant(&self, tenant: &str) -> usize {
        self.connection_info
            .get(tenant)
            .map(|e| e.value().len())
            .unwrap_or(0)
    }

    // topic rewrite rule
    pub fn add_topic_rewrite_rule(&self, topic_rewrite_rule: MqttTopicRewriteRule) {
        let inner_key = self.topic_rewrite_rule_inner_key(
            &topic_rewrite_rule.action,
            &topic_rewrite_rule.source_topic,
        );
        self.topic_rewrite_rule
            .entry(topic_rewrite_rule.tenant.clone())
            .or_default()
            .insert(inner_key, topic_rewrite_rule);
    }

    pub fn delete_topic_rewrite_rule(&self, tenant: &str, action: &str, source_topic: &str) {
        let inner_key = self.topic_rewrite_rule_inner_key(action, source_topic);
        if let Some(tenant_map) = self.topic_rewrite_rule.get(tenant) {
            tenant_map.remove(&inner_key);
        }
    }

    pub fn get_all_topic_rewrite_rule(&self) -> Vec<MqttTopicRewriteRule> {
        self.topic_rewrite_rule
            .iter()
            .flat_map(|tenant_entry| {
                tenant_entry
                    .value()
                    .iter()
                    .map(|rule| rule.value().clone())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn get_topic_rewrite_rules_by_tenant(&self, tenant: &str) -> Vec<MqttTopicRewriteRule> {
        self.topic_rewrite_rule
            .get(tenant)
            .map(|m| m.iter().map(|entry| entry.value().clone()).collect())
            .unwrap_or_default()
    }

    // topic_rewrite_new_name
    pub fn add_new_rewrite_name(&self, tenant: &str, topic_name: &str, new_topic_name: &str) {
        self.topic_rewrite_new_name
            .entry(tenant.to_string())
            .or_insert_with(DashMap::new)
            .insert(topic_name.to_string(), new_topic_name.to_string());
    }

    pub fn get_new_rewrite_name(&self, tenant: &str, topic_name: &str) -> Option<String> {
        self.topic_rewrite_new_name
            .get(tenant)
            .and_then(|inner| inner.get(topic_name).map(|v| v.clone()))
    }

    pub fn clear_rewrite_new_name(&self) {
        self.topic_rewrite_new_name.clear();
    }

    pub async fn is_re_calc_topic_rewrite(&self) -> bool {
        *self.re_calc_topic_rewrite.read().await
    }

    pub async fn set_re_calc_topic_rewrite(&self, flag: bool) {
        let mut data = self.re_calc_topic_rewrite.write().await;
        *data = flag;
    }

    pub fn login_success(&self, connect_id: u64, user_name: String) {
        for tenant_entry in self.connection_info.iter() {
            if let Some(mut conn) = tenant_entry.value().get_mut(&connect_id) {
                conn.login_success(user_name);
                return;
            }
        }
    }

    pub fn is_login(&self, connect_id: u64) -> bool {
        for tenant_entry in self.connection_info.iter() {
            if let Some(conn) = tenant_entry.value().get(&connect_id) {
                return conn.is_login;
            }
        }
        false
    }

    // topic alias
    pub fn get_topic_alias(&self, connect_id: u64, topic_alias: u16) -> Option<String> {
        for tenant_entry in self.connection_info.iter() {
            if let Some(conn) = tenant_entry.value().get(&connect_id) {
                return conn.topic_alias.get(&topic_alias).map(|v| v.clone());
            }
        }
        None
    }

    pub fn topic_alias_exists(&self, connect_id: u64, topic_alias: u16) -> bool {
        for tenant_entry in self.connection_info.iter() {
            if let Some(conn) = tenant_entry.value().get(&connect_id) {
                return conn.topic_alias.contains_key(&topic_alias);
            }
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
                for tenant_entry in self.connection_info.iter() {
                    if let Some(conn) = tenant_entry.value().get_mut(&connect_id) {
                        conn.topic_alias.insert(alias, topic_name.to_owned());
                        return;
                    }
                }
            }
        }
    }

    // heartbeat
    pub fn report_heartbeat(&self, client_id: String, live_time: ConnectionLiveTime) {
        self.heartbeat_data.insert(client_id, live_time);
    }

    pub fn get_heartbeat(&self, client_id: &str) -> Option<ConnectionLiveTime> {
        self.heartbeat_data.get(client_id).map(|data| data.clone())
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

    // blacklist
    pub fn add_blacklist(&self, blacklist: MqttAclBlackList) {
        self.acl_metadata.parse_mqtt_blacklist(blacklist);
    }

    pub fn remove_blacklist(&self, blacklist: MqttAclBlackList) {
        self.acl_metadata.remove_mqtt_blacklist(blacklist);
    }

    // topic_is_validator
    pub fn add_topic_is_validator(&self, topic_name: &str) {
        self.topic_is_validator.insert(topic_name.to_string(), true);
    }

    // Authn
    pub fn add_authn(&self, auth: AuthnConfig) {
        self.authn_list.insert(auth.uid.clone(), auth);
    }

    pub fn get_authn(&self) -> Vec<(String, AuthnConfig)> {
        let mut authn_list: Vec<(String, AuthnConfig)> = self
            .authn_list
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        authn_list.sort_by(|a, b| b.1.create_at.cmp(&a.1.create_at));
        authn_list
    }

    pub fn remove_authn(&self, uid: &str) {
        self.authn_list.remove(uid);
    }

    // key
    pub fn topic_rewrite_rule_inner_key(&self, action: &str, source_topic: &str) -> String {
        format!("{action}_{source_topic}")
    }

    pub fn add_auto_subscribe_rule(&self, rule: MqttAutoSubscribeRule) {
        self.auto_subscribe_rule
            .entry(rule.tenant.clone())
            .or_default()
            .insert(rule.topic.clone(), rule);
    }

    pub fn delete_auto_subscribe_rule(&self, tenant: &str, topic: &str) {
        if let Some(tenant_map) = self.auto_subscribe_rule.get(tenant) {
            tenant_map.remove(topic);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::tool::test_build_mqtt_cache_manager;

    use super::*;
    use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
    use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
    use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
    use common_base::tools::now_second;
    use metadata_struct::meta::node::BrokerNode;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use protocol::mqtt::common::{QoS, RetainHandling};

    #[tokio::test]
    async fn node_operations() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let node = BrokerNode {
            node_id: 1,
            node_ip: "127.0.0.1".to_string(),
            ..Default::default()
        };

        // add
        cache_manager.node_cache.add_node(node.clone());

        // get
        let nodes = cache_manager.node_cache.node_list();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_id, node.node_id);
        assert_eq!(nodes[0].node_ip, node.node_ip);

        // remove
        cache_manager.node_cache.remove_node(node.clone());

        // get again
        let nodes = cache_manager.node_cache.node_list();
        assert!(nodes.is_empty());
    }

    #[tokio::test]
    async fn user_info_operations() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let tenant = DEFAULT_TENANT.to_string();
        let user1 = MqttUser {
            tenant: tenant.clone(),
            username: "user1".to_string(),
            password: "password1".to_string(),
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };
        let user2 = MqttUser {
            tenant: tenant.clone(),
            username: "user2".to_string(),
            password: "password2".to_string(),
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };

        // add
        cache_manager.add_user(user1.clone());
        cache_manager.add_user(user2.clone());

        // get
        let user_info = cache_manager
            .user_info
            .get(&tenant)
            .and_then(|m| m.get(&user1.username).map(|u| u.clone()));
        assert!(user_info.is_some());
        assert_eq!(user_info.unwrap().username, user1.username);

        // remove user2
        cache_manager.del_user(&tenant, &user2.username);
        let user2_info = cache_manager
            .user_info
            .get(&tenant)
            .and_then(|m| m.get("user2").map(|u| u.clone()));
        assert!(user2_info.is_none());

        // remove user1
        cache_manager.del_user(&user1.tenant, &user1.username);

        // get again
        let user_info = cache_manager
            .user_info
            .get(&tenant)
            .and_then(|m| m.get(&user1.username).map(|u| u.clone()));
        assert!(user_info.is_none());
    }

    #[tokio::test]
    async fn session_info_operations() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let client_id = "test_client_session";
        let session = MqttSession {
            tenant: DEFAULT_TENANT.to_string(),
            client_id: client_id.to_string(),
            ..Default::default()
        };

        // add
        cache_manager.add_session(client_id, &session);

        // get
        let session_info = cache_manager.get_session_info(client_id);
        assert!(session_info.is_some());
        assert_eq!(session_info.unwrap().client_id, client_id);

        // update connect_id
        let new_connect_id = Some(12345);
        cache_manager.update_session_connect_id(client_id, new_connect_id);
        let updated_session = cache_manager.get_session_info(client_id).unwrap();
        assert_eq!(updated_session.connection_id, new_connect_id);

        // set connect_id to None
        cache_manager.update_session_connect_id(client_id, None);
        let updated_session_none = cache_manager.get_session_info(client_id).unwrap();
        assert!(updated_session_none.connection_id.is_none());

        // remove
        cache_manager.remove_session(client_id);

        // get again
        let session_info_after_remove = cache_manager.get_session_info(client_id);
        assert!(session_info_after_remove.is_none());
    }

    #[tokio::test]
    async fn connection_info_operations() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let connect_id = 12345;
        let client_id = "test_client_connection";
        let session = MqttSession {
            tenant: DEFAULT_TENANT.to_string(),
            client_id: client_id.to_string(),
            ..Default::default()
        };
        let conn = MQTTConnection {
            client_id: client_id.to_string(),
            ..Default::default()
        };

        // add
        cache_manager.add_session(client_id, &session);
        assert_eq!(cache_manager.get_connection_count(), 0);
        cache_manager.add_connection(connect_id, conn.clone());

        // get
        assert_eq!(cache_manager.get_connection_count(), 1);
        assert_eq!(cache_manager.get_connect_id(client_id), Some(connect_id));
        let conn_info = cache_manager.get_connection(connect_id);
        assert!(conn_info.is_some());
        assert_eq!(conn_info.unwrap().client_id, client_id);

        // login status
        assert!(!cache_manager.is_login(connect_id));
        cache_manager.login_success(connect_id, "test_user".to_string());
        assert!(cache_manager.is_login(connect_id));

        // remove
        cache_manager.remove_connection(connect_id);
        assert_eq!(cache_manager.get_connection_count(), 0);

        // get again
        let conn_info_after_remove = cache_manager.get_connection(connect_id);
        assert!(conn_info_after_remove.is_none());
    }

    #[tokio::test]
    async fn heartbeat_data_operations() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let client_id = "test_client_heartbeat";
        let live_time = ConnectionLiveTime {
            protocol: MqttProtocol::Mqtt3,
            keep_live: 60,
            heartbeat: now_second(),
        };

        // add
        cache_manager.report_heartbeat(client_id.to_string(), live_time);

        // get
        let heartbeat = cache_manager.heartbeat_data.get(client_id);
        assert!(heartbeat.is_some());
        assert_eq!(heartbeat.unwrap().keep_live, 60);

        // remove
        cache_manager.remove_heartbeat(client_id);

        // get again
        let heartbeat_after_remove = cache_manager.heartbeat_data.get(client_id);
        assert!(heartbeat_after_remove.is_none());
    }

    #[tokio::test]
    async fn topic_rewrite_rule_operations() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let rule = MqttTopicRewriteRule {
            tenant: DEFAULT_TENANT.to_string(),
            action: "publish".to_string(),
            source_topic: "source/topic".to_string(),
            dest_topic: "target/topic".to_string(),
            regex: "".to_string(),
            timestamp: 0,
        };

        // add
        cache_manager.add_topic_rewrite_rule(rule.clone());

        // get
        let rules = cache_manager.get_all_topic_rewrite_rule();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].source_topic, rule.source_topic);

        // remove
        cache_manager.delete_topic_rewrite_rule(&rule.tenant, &rule.action, &rule.source_topic);

        // get again
        let rules_after_remove = cache_manager.get_all_topic_rewrite_rule();
        assert!(rules_after_remove.is_empty());
    }

    #[tokio::test]
    async fn auto_subscribe_rule_operations() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let rule = MqttAutoSubscribeRule {
            tenant: "tenant-1".to_string(),
            topic: "auto/sub/topic".to_string(),
            qos: QoS::AtLeastOnce,
            no_local: false,
            retain_as_published: false,
            retained_handling: RetainHandling::OnEverySubscribe,
        };

        // add
        cache_manager.add_auto_subscribe_rule(rule.clone());

        // get
        let rule_info = cache_manager
            .auto_subscribe_rule
            .get(&rule.tenant)
            .and_then(|m| m.get(&rule.topic).map(|v| v.clone()));
        println!("{rule_info:?}");
        assert!(rule_info.is_some());
        assert_eq!(rule_info.unwrap().topic, rule.topic);

        // remove
        cache_manager.delete_auto_subscribe_rule(&rule.tenant, &rule.topic);

        // get again
        let rule_info_after_remove = cache_manager
            .auto_subscribe_rule
            .get(&rule.tenant)
            .and_then(|m| m.get(&rule.topic).map(|v| v.clone()));
        assert!(rule_info_after_remove.is_none());
    }

    #[tokio::test]
    async fn topic_alias_operations() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let client_id = "test_client_alias";
        let connect_id = 1;
        let session = MqttSession {
            tenant: DEFAULT_TENANT.to_string(),
            client_id: client_id.to_string(),
            ..Default::default()
        };
        let conn = MQTTConnection {
            client_id: client_id.to_string(),
            ..Default::default()
        };
        cache_manager.add_session(client_id, &session);
        cache_manager.add_connection(connect_id, conn);

        let topic_name = "test/alias/topic";
        let topic_alias = 10;

        // get non-existent
        assert!(!cache_manager.topic_alias_exists(connect_id, topic_alias));
        assert!(cache_manager
            .get_topic_alias(connect_id, topic_alias)
            .is_none());

        // add
        let properties = Some(PublishProperties {
            topic_alias: Some(topic_alias),
            ..Default::default()
        });
        cache_manager.add_topic_alias(connect_id, topic_name, &properties);

        // get existent
        assert!(cache_manager.topic_alias_exists(connect_id, topic_alias));
        assert_eq!(
            cache_manager.get_topic_alias(connect_id, topic_alias),
            Some(topic_name.to_string())
        );
    }

    #[tokio::test]
    async fn acl_operations() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let tenant = DEFAULT_TENANT.to_string();
        let user_acl = MqttAcl {
            name: "acl-user-test".to_string(),
            desc: String::new(),
            tenant: tenant.clone(),
            resource_type: MqttAclResourceType::User,
            resource_name: "test_user_acl".to_string(),
            topic: "#".to_string(),
            ip: "127.0.0.1".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Allow,
        };
        let client_acl = MqttAcl {
            name: "acl-client-test".to_string(),
            desc: String::new(),
            tenant: tenant.clone(),
            resource_type: MqttAclResourceType::ClientId,
            resource_name: "test_client_acl".to_string(),
            topic: "test/topic".to_string(),
            ip: "127.0.0.1".to_string(),
            action: MqttAclAction::Subscribe,
            permission: MqttAclPermission::Allow,
        };

        // add
        cache_manager.add_acl(user_acl.clone());
        cache_manager.add_acl(client_acl.clone());
        assert!(cache_manager
            .acl_metadata
            .acl_user
            .get(&tenant)
            .map(|m| m.contains_key("test_user_acl"))
            .unwrap_or(false));
        assert!(cache_manager
            .acl_metadata
            .acl_client_id
            .get(&tenant)
            .map(|m| m.contains_key("test_client_acl"))
            .unwrap_or(false));

        // remove client_acl
        cache_manager.remove_acl(client_acl);
        assert!(!cache_manager
            .acl_metadata
            .acl_client_id
            .get(&tenant)
            .map(|m| m.contains_key("test_client_acl"))
            .unwrap_or(false));

        // remove user_acl
        cache_manager.remove_acl(user_acl);
        assert!(!cache_manager
            .acl_metadata
            .acl_user
            .get(&tenant)
            .map(|m| m.contains_key("test_user_acl"))
            .unwrap_or(false));
    }

    #[tokio::test]
    async fn blacklist_operations() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let tenant = DEFAULT_TENANT.to_string();
        let blacklist = MqttAclBlackList {
            name: "bl-cache-test".to_string(),
            tenant: tenant.clone(),
            blacklist_type: MqttAclBlackListType::ClientId,
            resource_name: "blacklist_client".to_string(),
            end_time: 0,
            desc: "".to_string(),
        };

        // add
        cache_manager.add_blacklist(blacklist.clone());
        assert!(cache_manager
            .acl_metadata
            .blacklist_client_id
            .get(&tenant)
            .map(|m| m.contains_key("blacklist_client"))
            .unwrap_or(false));

        // remove
        cache_manager.remove_blacklist(blacklist);
        assert!(!cache_manager
            .acl_metadata
            .blacklist_client_id
            .get(&tenant)
            .map(|m| m.contains_key("blacklist_client"))
            .unwrap_or(false));
    }
}
