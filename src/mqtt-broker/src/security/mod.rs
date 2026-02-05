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
use crate::security::login::http::http_check_login;
use crate::security::login::jwt::jwt_check_login;
use crate::security::login::mysql::mysql_check_login;
use crate::security::login::plaintext::plaintext_check_login;
use crate::security::login::postgresql::postgresql_check_login;
use crate::security::login::redis::redis_check_login;
use crate::security::storage::storage_trait::AuthStorageAdapter;
use crate::security::storage::AuthType;
use crate::subscribe::common::get_sub_topic_name_list;
use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use common_config::broker::broker_config;
use common_config::security::{AuthnConfig, StorageConfig};
use common_metrics::mqtt::auth::{
    record_mqtt_acl_failed, record_mqtt_acl_success, record_mqtt_blacklist_blocked,
};
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::user::MqttUser;
use protocol::mqtt::common::{ConnectProperties, Login, QoS, Subscribe};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use storage::http::HttpAuthStorageAdapter;
use storage::meta::MetaServiceAuthStorageAdapter;
use storage::mysql::MySQLAuthStorageAdapter;
use storage::postgresql::PostgresqlAuthStorageAdapter;
use storage::redis::RedisAuthStorageAdapter;

pub mod auth;
pub mod login;
pub mod storage;

#[derive(Clone)]
pub struct AuthDriver {
    cache_manager: Arc<MQTTCacheManager>,
    driver: Arc<dyn AuthStorageAdapter + Send + 'static + Sync>,
}

impl AuthDriver {
    pub fn new(cache_manager: Arc<MQTTCacheManager>, client_pool: Arc<ClientPool>) -> AuthDriver {
        let conf = broker_config();

        let driver = match build_auth_driver(client_pool, &conf.mqtt_auth_config.authn_config) {
            Ok(driver) => driver,
            Err(e) => {
                panic!("{}, auth config:{:?}", e, conf.mqtt_auth_config);
            }
        };

        AuthDriver {
            cache_manager,
            driver,
        }
    }

    // Permission: Allow && Deny
    pub async fn auth_login_check(
        &self,
        login: &Option<Login>,
        _connect_properties: &Option<ConnectProperties>,
        _socket_addr: &SocketAddr,
        client_id: Option<&str>,
    ) -> Result<bool, MqttBrokerError> {
        let cluster = self.cache_manager.broker_cache.get_cluster_config().await;

        if cluster.mqtt_security.secret_free_login {
            return Ok(true);
        }

        if let Some(info) = login {
            let conf = broker_config();

            // according to auth_type to select authentication method
            match conf.mqtt_auth_config.authn_config.authn_type.as_str() {
                "password_based" => {
                    // get password authentication configuration
                    if let Some(password_based_config) =
                        &conf.mqtt_auth_config.authn_config.password_based_config
                    {
                        let password_config = &password_based_config.password_config;
                        let storage_config = &password_based_config.storage_config;

                        // according to storage type to select corresponding verification function
                        match storage_config.storage_type.as_str() {
                            "mysql" => {
                                mysql_check_login(
                                    &self.driver,
                                    &self.cache_manager,
                                    password_config,
                                    &info.username,
                                    &info.password,
                                )
                                .await
                            }
                            "postgresql" => {
                                postgresql_check_login(
                                    &self.driver,
                                    &self.cache_manager,
                                    password_config,
                                    &info.username,
                                    &info.password,
                                )
                                .await
                            }
                            "redis" => {
                                redis_check_login(
                                    &self.driver,
                                    &self.cache_manager,
                                    password_config,
                                    &info.username,
                                    &info.password,
                                )
                                .await
                            }
                            "placement" => {
                                plaintext_check_login(
                                    &self.driver,
                                    &self.cache_manager,
                                    &info.username,
                                    &info.password,
                                )
                                .await
                            }
                            "http" => {
                                // HTTP authentication as a storage type under password-based
                                if let Some(http_config) = &storage_config.http_config {
                                    let client_id_str = client_id.unwrap_or("unknown");
                                    http_check_login(
                                        &self.driver,
                                        &self.cache_manager,
                                        http_config,
                                        &info.username,
                                        &info.password,
                                        client_id_str,
                                        &_socket_addr.ip().to_string(), // source_ip
                                    )
                                    .await
                                } else {
                                    Err(MqttBrokerError::HttpConfigNotFound)
                                }
                            }
                            _ => Err(MqttBrokerError::UnavailableStorageType),
                        }
                    } else {
                        Err(MqttBrokerError::PasswordConfigNotFound)
                    }
                }
                "jwt" => {
                    // JWT authentication
                    if let Some(jwt_config) = &conf.mqtt_auth_config.authn_config.jwt_config {
                        jwt_check_login(
                            &self.cache_manager,
                            jwt_config,
                            &info.username,
                            &info.password,
                        )
                        .await
                    } else {
                        Err(MqttBrokerError::JwtConfigNotFound)
                    }
                }
                _ => Err(MqttBrokerError::UnsupportedAuthType(
                    conf.mqtt_auth_config.authn_config.authn_type.clone(),
                )),
            }
        } else {
            Ok(false)
        }
    }

    pub async fn auth_connect_check(&self, connection: &MQTTConnection) -> bool {
        // default true if blacklist check fails
        is_blacklist(&self.cache_manager, connection).unwrap_or(true)
    }

    pub async fn auth_publish_check(
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

        // check blacklist
        // default true if blacklist check fails
        if is_blacklist(&self.cache_manager, connection).unwrap_or(true) {
            record_mqtt_blacklist_blocked();
            return Err(MqttBrokerError::NotBlacklistAuth);
        }

        Ok(())
    }

    pub async fn auth_subscribe_check(
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
        self.driver.read_all_user().await
    }

    pub async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        self.driver.read_all_acl().await
    }

    pub async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        self.driver.read_all_blacklist().await
    }

    pub async fn save_user(&self, user_info: MqttUser) -> ResultMqttBrokerError {
        let username = user_info.username.clone();
        if let Some(_user) = self.cache_manager.user_info.get(&username) {
            return Err(MqttBrokerError::UserAlreadyExist);
        }
        self.cache_manager.add_user(user_info.clone());
        self.driver.save_user(user_info).await
    }

    pub async fn delete_user(&self, username: String) -> ResultMqttBrokerError {
        if self.cache_manager.user_info.get(&username).is_none() {
            return Err(MqttBrokerError::UserDoesNotExist);
        }
        self.driver.delete_user(username.clone()).await?;
        self.cache_manager.del_user(username.clone());
        Ok(())
    }

    pub async fn update_user_cache(&self) -> ResultMqttBrokerError {
        let all_users: DashMap<String, MqttUser> = self.driver.read_all_user().await?;

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
        self.driver.save_acl(acl).await
    }

    pub async fn delete_acl(&self, acl: MqttAcl) -> ResultMqttBrokerError {
        self.driver.delete_acl(acl.clone()).await?;
        self.cache_manager.remove_acl(acl.clone());
        Ok(())
    }

    pub async fn update_acl_cache(&self) -> ResultMqttBrokerError {
        let all_acls: Vec<MqttAcl> = self.driver.read_all_acl().await?;

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
        self.driver.save_blacklist(blacklist).await
    }

    pub async fn delete_blacklist(&self, blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        self.driver.delete_blacklist(blacklist.clone()).await?;
        self.cache_manager.remove_blacklist(blacklist.clone());
        Ok(())
    }

    pub async fn update_blacklist_cache(&self) -> ResultMqttBrokerError {
        let all_blacklist = self.driver.read_all_blacklist().await?;

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
    match authn_config.authn_type.as_str() {
        "password_based" => {
            if let Some(password_based_config) = &authn_config.password_based_config {
                let storage_config = &password_based_config.storage_config;
                build_storage_driver(client_pool, storage_config)
            } else {
                Err(MqttBrokerError::PasswordConfigNotFound)
            }
        }
        "jwt" => {
            // JWT authentication doesn't need specific storage adapter (pure token validation)
            // But we still create a minimal adapter to maintain API compatibility
            let driver = MetaServiceAuthStorageAdapter::new(client_pool);
            Ok(Arc::new(driver))
        }
        _ => Err(MqttBrokerError::UnsupportedAuthType(
            authn_config.authn_type.clone(),
        )),
    }
}

fn build_storage_driver(
    client_pool: Arc<ClientPool>,
    storage_config: &StorageConfig,
) -> Result<Arc<dyn AuthStorageAdapter + Send + 'static + Sync>, MqttBrokerError> {
    let storage_type = AuthType::from_str(&storage_config.storage_type)
        .map_err(|_| MqttBrokerError::UnavailableStorageType)?;

    match storage_type {
        AuthType::Placement => {
            // Placement adapter only needs client_pool parameter
            let driver = MetaServiceAuthStorageAdapter::new(client_pool);
            Ok(Arc::new(driver))
        }
        AuthType::Mysql => {
            if let Some(mysql_config) = &storage_config.mysql_config {
                let driver = MySQLAuthStorageAdapter::new(mysql_config.clone());
                Ok(Arc::new(driver))
            } else {
                Err(MqttBrokerError::CommonError(
                    "Mysql config not found".to_string(),
                ))
            }
        }
        AuthType::Postgresql => {
            if let Some(postgres_config) = &storage_config.postgres_config {
                let driver = PostgresqlAuthStorageAdapter::new(postgres_config.clone());
                Ok(Arc::new(driver))
            } else {
                Err(MqttBrokerError::CommonError(
                    "Postgres config not found".to_string(),
                ))
            }
        }
        AuthType::Redis => {
            if let Some(redis_config) = &storage_config.redis_config {
                let driver = RedisAuthStorageAdapter::new(redis_config.clone());
                Ok(Arc::new(driver))
            } else {
                Err(MqttBrokerError::CommonError(
                    "Redis config not found".to_string(),
                ))
            }
        }
        AuthType::Http => {
            // HTTP storage requires special handling due to its unique implementation
            if let Some(http_config) = &storage_config.http_config {
                let driver = HttpAuthStorageAdapter::new(http_config.clone());
                Ok(Arc::new(driver))
            } else {
                Err(MqttBrokerError::HttpConfigNotFound)
            }
        }
        _ => Err(MqttBrokerError::UnavailableStorageType),
    }
}
