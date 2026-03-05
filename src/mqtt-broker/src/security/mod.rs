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
use common_base::tools::now_millis;
use common_metrics::mqtt::auth::{record_mqtt_acl_failed, record_mqtt_acl_success};
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::auth::authn_config::AuthnConfig;
use metadata_struct::mqtt::auth::authn_config::LoginAuthEnum;
use metadata_struct::mqtt::auth::password::PasswordBasedConfig;
use metadata_struct::mqtt::auth::storage::StorageConfig;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::user::MqttUser;
use protocol::mqtt::common::{ConnectProperties, Login, QoS, Subscribe};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

pub mod auth;
pub mod login;
pub mod storage;

const STORAGE_DRIVER_REBUILD_MS: u128 = 5000;

#[derive(Clone)]
struct CachedStorageDriver {
    driver: Arc<dyn AuthStorageAdapter + Send + Sync>,
    build_at_ms: u128,
}

#[derive(Clone)]
pub struct AuthManager {
    cache_manager: Arc<MQTTCacheManager>,
    storage_drivers: Arc<DashMap<String, CachedStorageDriver>>,
}

impl AuthManager {
    pub fn new(cache_manager: Arc<MQTTCacheManager>) -> AuthManager {
        AuthManager {
            cache_manager,
            storage_drivers: Arc::new(DashMap::new()),
        }
    }

    fn authn_list_with_default(&self) -> Vec<(String, AuthnConfig)> {
        let mut authn_list = self.cache_manager.get_authn();
        if authn_list.is_empty() {
            let default_authn = AuthnConfig {
                uid: "inner_default".to_string(),
                authn_type: "password_based".to_string(),
                config: LoginAuthEnum::PasswordBased(Box::new(PasswordBasedConfig {
                    storage_config: StorageConfig {
                        storage_type: "meta".to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                })),
                create_at: 0,
            };
            authn_list.push((default_authn.uid.clone(), default_authn));
        }
        authn_list
    }

    fn get_or_build_storage_driver(
        &self,
        authn_id: &str,
        storage_config: &StorageConfig,
    ) -> Result<Arc<dyn AuthStorageAdapter + Send + 'static + Sync>, MqttBrokerError> {
        let now = now_millis();
        if let Some(cached) = self.storage_drivers.get(authn_id) {
            if now.saturating_sub(cached.build_at_ms) <= STORAGE_DRIVER_REBUILD_MS {
                return Ok(cached.driver.clone());
            }
        }

        let driver = build_storage_driver(storage_config)?;
        self.storage_drivers.insert(
            authn_id.to_string(),
            CachedStorageDriver {
                driver: driver.clone(),
                build_at_ms: now,
            },
        );
        Ok(driver)
    }

    fn password_based_drivers(
        &self,
    ) -> Result<Vec<Arc<dyn AuthStorageAdapter + Send + 'static + Sync>>, MqttBrokerError> {
        let mut drivers = Vec::new();
        for (authn_id, authn) in self.authn_list_with_default() {
            if let LoginAuthEnum::PasswordBased(config) = authn.config {
                let driver = self.get_or_build_storage_driver(&authn_id, &config.storage_config)?;
                drivers.push(driver);
            }
        }
        Ok(drivers)
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

        if let Some((_, authn)) = self.authn_list_with_default().into_iter().next() {
            let login_type = LoginType::from_str(&authn.authn_type)
                .map_err(|_| MqttBrokerError::UnsupportedAuthType(authn.authn_type.clone()))?;

            return match login_type {
                LoginType::PasswordBased => {
                    if let Some(user_info) = login {
                        Ok(password_check_by_login(
                            &self.cache_manager,
                            &user_info.username,
                            &user_info.password,
                        ))
                    } else {
                        Ok(false)
                    }
                }
                LoginType::Jwt => Ok(false),
            };
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

    // read all
    pub async fn read_all_user(&self) -> Result<HashMap<String, MqttUser>, MqttBrokerError> {
        let mut results = HashMap::new();
        for driver in self.password_based_drivers()? {
            let list = driver.read_all_user().await?;
            for user in list.iter() {
                results.insert(user.username.clone(), user.clone());
            }
        }
        Ok(results)
    }

    pub async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        let mut results = Vec::new();
        for driver in self.password_based_drivers()? {
            let list = driver.read_all_acl().await?;
            for acl in list.iter() {
                results.push(acl.clone());
            }
        }
        Ok(results)
    }

    pub async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        let mut results = Vec::new();
        for driver in self.password_based_drivers()? {
            let list = driver.read_all_blacklist().await?;
            for blacklist in list.iter() {
                results.push(blacklist.clone());
            }
        }
        Ok(results)
    }

    pub async fn update_user_cache(&self) -> ResultMqttBrokerError {
        let all_users: HashMap<String, MqttUser> = self.read_all_user().await?;

        for user in all_users.values() {
            self.cache_manager.add_user(user.clone());
        }

        Ok(())
    }

    // ACL
    pub async fn update_acl_cache(&self) -> ResultMqttBrokerError {
        let all_acls: Vec<MqttAcl> = self.read_all_acl().await?;

        for acl in all_acls.iter() {
            self.cache_manager.add_acl(acl.to_owned());
        }

        Ok(())
    }

    // BlackList
    pub async fn update_blacklist_cache(&self) -> ResultMqttBrokerError {
        let all_blacklist = self.read_all_blacklist().await?;

        for acl in all_blacklist.iter() {
            self.cache_manager.add_blacklist(acl.to_owned());
        }

        Ok(())
    }
}
