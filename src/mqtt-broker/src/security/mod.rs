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

use crate::core::cache::MQTTCacheManager;
use crate::core::error::MqttBrokerError;
use crate::core::tool::ResultMqttBrokerError;
use crate::security::auth::blacklist::is_blacklist;
use crate::security::auth::is_allow_acl;
use crate::security::login::password::password_check_by_login;
use crate::security::login::LoginType;
use crate::security::storage::build_storage_driver;
use crate::security::storage::storage_trait::AuthStorageAdapter;
use crate::subscribe::common::get_sub_topic_name_list;
use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use common_config::broker::broker_config;
use common_config::security::AuthnConfig;
use common_metrics::mqtt::auth::{record_mqtt_acl_failed, record_mqtt_acl_success};
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::user::MqttUser;
use protocol::mqtt::common::{ConnectProperties, Login, QoS, Subscribe};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use storage::meta::MetaServiceAuthStorageAdapter;

pub mod auth;
pub mod login;
pub mod storage;

#[derive(Clone)]
pub struct AuthManager {
    cache_manager: Arc<MQTTCacheManager>,
    storage_driver: Arc<dyn AuthStorageAdapter + Send + 'static + Sync>,
}

impl AuthManager {
    pub fn new(cache_manager: Arc<MQTTCacheManager>, client_pool: Arc<ClientPool>) -> AuthManager {
        let conf = broker_config();

        let driver = match build_auth_driver(client_pool, &conf.mqtt_auth_config.authn_config) {
            Ok(driver) => driver,
            Err(e) => {
                panic!("{}, auth config:{:?}", e, conf.mqtt_auth_config);
            }
        };

        AuthManager {
            cache_manager,
            storage_driver: driver,
        }
    }

    // Permission: Allow && Deny
    pub async fn login_check(
        &self,
        login: &Option<Login>,
        _connect_properties: &Option<ConnectProperties>,
    ) -> Result<bool, MqttBrokerError> {
        let cluster = self.cache_manager.broker_cache.get_cluster_config().await;

        if cluster.mqtt_security.secret_free_login {
            return Ok(true);
        }

        if let Some(info) = login {
            let conf = broker_config();
            let login_type = LoginType::from_str(&conf.mqtt_auth_config.authn_config.authn_type)
                .map_err(|_| {
                    MqttBrokerError::UnsupportedAuthType(
                        conf.mqtt_auth_config.authn_config.authn_type.clone(),
                    )
                })?;

            // according to auth_type to select authentication method
            match login_type {
                LoginType::PasswordBased => {
                    return Ok(password_check_by_login(
                        &self.cache_manager,
                        &info.username,
                        &info.password,
                    ));
                }
                LoginType::Jwt => {
                    // JWT authentication
                    // if let Some(jwt_config) = &conf.mqtt_auth_config.authn_config.jwt_config {
                    //     jwt_check_login(
                    //         &self.cache_manager,
                    //         jwt_config,
                    //         &info.username,
                    //         &info.password,
                    //     )
                    //     .await
                    // } else {
                    //     Err(MqttBrokerError::JwtConfigNotFound)
                    // }
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub async fn connect_check(
        &self,
        client_id: &str,
        source_ip_addr: &str,
        login: &Option<Login>,
    ) -> bool {
        // default true if blacklist check fails
        is_blacklist(&self.cache_manager, client_id, source_ip_addr, login).unwrap_or(true)
    }

    pub async fn publish_check(
        &self,
        connection: &MQTTConnection,
        topic_name: &str,
        retain: bool,
        qos: QoS,
    ) -> Result<(), MqttBrokerError> {
        if !is_allow_acl(
            &self.cache_manager,
            connection,
            topic_name,
            MqttAclAction::Publish,
            retain,
            qos,
        ) {
            record_mqtt_acl_failed();
            return Err(MqttBrokerError::NotAclAuth(topic_name.to_string()));
        }
        record_mqtt_acl_success();

        Ok(())
    }

    pub async fn subscribe_check(
        &self,
        connection: &MQTTConnection,
        subscribe: &Subscribe,
    ) -> bool {
        for filter in subscribe.filters.iter() {
            let topic_list = get_sub_topic_name_list(&self.cache_manager, &filter.path).await;
            for topic_name in topic_list {
                if !is_allow_acl(
                    &self.cache_manager,
                    connection,
                    &topic_name,
                    MqttAclAction::Subscribe,
                    false,
                    filter.qos,
                ) {
                    return false;
                }
            }
        }
        true
    }

    // User
    pub async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        self.storage_driver.read_all_user().await
    }

    pub async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        self.storage_driver.read_all_acl().await
    }

    pub async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        self.storage_driver.read_all_blacklist().await
    }

    pub async fn save_user(&self, user_info: MqttUser) -> ResultMqttBrokerError {
        let username = user_info.username.clone();
        if let Some(_user) = self.cache_manager.user_info.get(&username) {
            return Err(MqttBrokerError::UserAlreadyExist);
        }
        self.cache_manager.add_user(user_info.clone());
        self.storage_driver.save_user(user_info).await
    }

    pub async fn delete_user(&self, username: String) -> ResultMqttBrokerError {
        if self.cache_manager.user_info.get(&username).is_none() {
            return Err(MqttBrokerError::UserDoesNotExist);
        }
        self.storage_driver.delete_user(username.clone()).await?;
        self.cache_manager.del_user(username.clone());
        Ok(())
    }

    pub async fn update_user_cache(&self) -> ResultMqttBrokerError {
        let all_users: DashMap<String, MqttUser> = self.storage_driver.read_all_user().await?;

        for entry in all_users.iter() {
            let user = entry.value().clone();
            self.cache_manager.add_user(user);
        }

        let db_usernames: HashSet<String> =
            all_users.iter().map(|user| user.key().clone()).collect();
        self.cache_manager.retain_users(db_usernames);

        Ok(())
    }

    // ACL
    pub async fn save_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
        self.cache_manager.add_acl(acl.clone());
        self.storage_driver.save_acl(acl).await
    }

    pub async fn delete_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
        self.storage_driver.delete_acl(acl.clone()).await?;
        self.cache_manager.remove_acl(acl.clone());
        Ok(())
    }

    pub async fn update_acl_cache(&self) -> ResultMqttBrokerError {
        let all_acls: Vec<MqttAcl> = self.storage_driver.read_all_acl().await?;

        for acl in all_acls.iter() {
            self.cache_manager.add_acl(acl.to_owned());
        }

        let mut user_acl = HashSet::new();
        let mut client_acl = HashSet::new();

        for acl in all_acls {
            match acl.resource_type {
                MqttAclResourceType::User => user_acl.insert(acl.resource_name.clone()),
                MqttAclResourceType::ClientId => client_acl.insert(acl.resource_name.clone()),
            };
        }
        self.cache_manager.retain_acls(user_acl, client_acl);

        Ok(())
    }

    // BlackList
    pub async fn save_blacklist(&self, blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        self.cache_manager.add_blacklist(blacklist.clone());
        self.storage_driver.save_blacklist(blacklist).await
    }

    pub async fn delete_blacklist(&self, blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        self.storage_driver
            .delete_blacklist(blacklist.clone())
            .await?;
        self.cache_manager.remove_blacklist(blacklist.clone());
        Ok(())
    }

    pub async fn update_blacklist_cache(&self) -> ResultMqttBrokerError {
        let all_blacklist = self.storage_driver.read_all_blacklist().await?;

        for acl in all_blacklist.iter() {
            self.cache_manager.add_blacklist(acl.to_owned());
        }

        Ok(())
    }
}

fn build_auth_driver(
    client_pool: Arc<ClientPool>,
    authn_config: &AuthnConfig,
) -> Result<Arc<dyn AuthStorageAdapter + Send + 'static + Sync>, MqttBrokerError> {
    let login_type = LoginType::from_str(&authn_config.authn_type)
        .map_err(|_| MqttBrokerError::UnsupportedAuthType(authn_config.authn_type.clone()))?;

    match login_type {
        LoginType::PasswordBased => {
            if let Some(password_based_config) = &authn_config.password_based_config {
                let storage_config = &password_based_config.storage_config;
                build_storage_driver(client_pool, storage_config)
            } else {
                Err(MqttBrokerError::PasswordConfigNotFound)
            }
        }
        LoginType::Jwt => {
            // JWT authentication doesn't need specific storage adapter (pure token validation)
            // But we still create a minimal adapter to maintain API compatibility
            let driver = MetaServiceAuthStorageAdapter::new(client_pool);
            Ok(Arc::new(driver))
        }
    }
}
