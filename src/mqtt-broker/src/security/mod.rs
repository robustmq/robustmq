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

use std::collections::HashSet;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use acl::is_allow_acl;
use axum::async_trait;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::config::common::Auth;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use login::plaintext::Plaintext;
use login::Authentication;
use metadata_struct::acl::mqtt_acl::{MqttAcl, MqttAclAction, MqttAclResourceType};
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::user::MqttUser;
use mysql::MySQLAuthStorageAdapter;
use placement::PlacementAuthStorageAdapter;
use protocol::mqtt::common::{ConnectProperties, Login, QoS, Subscribe};
use storage_adapter::StorageType;

use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::subscribe::sub_common::get_sub_topic_id_list;

pub mod acl;
pub mod login;
pub mod mysql;
pub mod placement;
pub mod redis;

#[async_trait]
pub trait AuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError>;

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError>;

    async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError>;

    async fn get_user(&self, username: String) -> Result<Option<MqttUser>, MqttBrokerError>;

    async fn save_user(&self, user_info: MqttUser) -> Result<(), MqttBrokerError>;

    async fn delete_user(&self, username: String) -> Result<(), MqttBrokerError>;

    async fn save_acl(&self, acl: MqttAcl) -> Result<(), MqttBrokerError>;

    async fn delete_acl(&self, acl: MqttAcl) -> Result<(), MqttBrokerError>;

    async fn save_blacklist(&self, blacklist: MqttAclBlackList) -> Result<(), MqttBrokerError>;

    async fn delete_blacklist(&self, blacklist: MqttAclBlackList) -> Result<(), MqttBrokerError>;
}

pub struct AuthDriver {
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
    driver: Arc<dyn AuthStorageAdapter + Send + 'static + Sync>,
}

impl AuthDriver {
    pub fn new(cache_manager: Arc<CacheManager>, client_pool: Arc<ClientPool>) -> AuthDriver {
        let conf = broker_mqtt_conf();
        let driver = match build_driver(client_pool.clone(), conf.auth.clone()) {
            Ok(driver) => driver,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        AuthDriver {
            cache_manager,
            driver,
            client_pool,
        }
    }

    pub fn update_driver(&mut self, auth: Auth) -> Result<(), MqttBrokerError> {
        let driver = build_driver(self.client_pool.clone(), auth)?;
        self.driver = driver;
        Ok(())
    }

    pub async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        self.driver.read_all_user().await
    }

    pub async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        self.driver.read_all_acl().await
    }

    pub async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        self.driver.read_all_blacklist().await
    }

    pub async fn save_user(&self, user_info: MqttUser) -> Result<(), MqttBrokerError> {
        let username = user_info.username.clone();
        if let Some(_user) = self.cache_manager.user_info.get(&username) {
            return Err(MqttBrokerError::UserAlreadyExist);
        }
        self.cache_manager.add_user(user_info.clone());
        self.driver.save_user(user_info).await
    }

    pub async fn delete_user(&self, username: String) -> Result<(), MqttBrokerError> {
        if self.cache_manager.user_info.get(&username).is_none() {
            return Err(MqttBrokerError::UserDoesNotExist);
        }
        self.driver.delete_user(username.clone()).await?;
        self.cache_manager.del_user(username.clone());
        Ok(())
    }

    pub async fn update_user_cache(&self) -> Result<(), MqttBrokerError> {
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

    pub async fn check_login_auth(
        &self,
        login: &Option<Login>,
        _: &Option<ConnectProperties>,
        _: &SocketAddr,
    ) -> Result<bool, MqttBrokerError> {
        let cluster = self.cache_manager.get_cluster_info();

        if cluster.security.secret_free_login {
            return Ok(true);
        }

        if let Some(info) = login {
            return self
                .plaintext_check_login(&info.username, &info.password)
                .await;
        }

        Ok(false)
    }

    pub async fn save_acl(&self, acl: MqttAcl) -> Result<(), MqttBrokerError> {
        self.cache_manager.add_acl(acl.clone());
        self.driver.save_acl(acl).await
    }

    pub async fn delete_acl(&self, acl: MqttAcl) -> Result<(), MqttBrokerError> {
        self.driver.delete_acl(acl.clone()).await?;
        self.cache_manager.remove_acl(acl.clone());
        Ok(())
    }

    pub async fn update_acl_cache(&self) -> Result<(), MqttBrokerError> {
        let all_acls: Vec<MqttAcl> = self.driver.read_all_acl().await?;

        for acl in all_acls.iter() {
            self.cache_manager.add_acl(acl.to_owned());
        }

        let mut user_acl = HashSet::new();
        let mut client_acl = HashSet::new();

        for acl in all_acls.clone() {
            match acl.resource_type {
                MqttAclResourceType::User => user_acl.insert(acl.resource_name.clone()),
                MqttAclResourceType::ClientId => client_acl.insert(acl.resource_name.clone()),
            };
        }
        self.cache_manager.retain_acls(user_acl, client_acl);

        Ok(())
    }

    pub async fn save_blacklist(&self, blacklist: MqttAclBlackList) -> Result<(), MqttBrokerError> {
        self.cache_manager.add_blacklist(blacklist.clone());
        self.driver.save_blacklist(blacklist).await
    }

    pub async fn delete_blacklist(
        &self,
        blacklist: MqttAclBlackList,
    ) -> Result<(), MqttBrokerError> {
        self.driver.delete_blacklist(blacklist.clone()).await?;
        self.cache_manager.remove_blacklist(blacklist.clone());
        Ok(())
    }

    pub async fn allow_publish(
        &self,
        connection: &MQTTConnection,
        topic_name: &str,
        retain: bool,
        qos: QoS,
    ) -> bool {
        is_allow_acl(
            &self.cache_manager,
            connection,
            topic_name,
            MqttAclAction::Publish,
            retain,
            qos,
        )
    }

    pub async fn allow_subscribe(
        &self,
        connection: &MQTTConnection,
        subscribe: &Subscribe,
    ) -> bool {
        for filter in subscribe.filters.clone() {
            let topic_list = get_sub_topic_id_list(&self.cache_manager, &filter.path).await;
            for topic in topic_list {
                if !is_allow_acl(
                    &self.cache_manager,
                    connection,
                    &topic,
                    MqttAclAction::Publish,
                    false,
                    filter.qos,
                ) {
                    return false;
                }
            }
        }
        true
    }

    async fn plaintext_check_login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<bool, MqttBrokerError> {
        let plaintext = Plaintext::new(
            username.to_owned(),
            password.to_owned(),
            self.cache_manager.clone(),
        );
        match plaintext.apply().await {
            Ok(flag) => {
                if flag {
                    return Ok(true);
                }
            }
            Err(e) => {
                // If the user does not exist, try to get the user information from the storage layer
                if e.to_string() == MqttBrokerError::UserDoesNotExist.to_string() {
                    return self.try_get_check_user_by_driver(username).await;
                }
                return Err(e);
            }
        }
        Ok(false)
    }

    async fn try_get_check_user_by_driver(&self, username: &str) -> Result<bool, MqttBrokerError> {
        if let Some(user) = self.driver.get_user(username.to_owned()).await? {
            self.cache_manager.add_user(user.clone());

            let plaintext = Plaintext::new(
                user.username.clone(),
                user.password.clone(),
                self.cache_manager.clone(),
            );

            if plaintext.apply().await? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

pub fn build_driver(
    client_pool: Arc<ClientPool>,
    auth: Auth,
) -> Result<Arc<dyn AuthStorageAdapter + Send + 'static + Sync>, MqttBrokerError> {
    let storage_type = StorageType::from_str(&auth.storage_type)
        .map_err(|_| MqttBrokerError::UnavailableStorageType)?;
    if matches!(storage_type, StorageType::Placement) {
        let driver = PlacementAuthStorageAdapter::new(client_pool);
        return Ok(Arc::new(driver));
    }

    if matches!(storage_type, StorageType::Mysql) {
        let driver = MySQLAuthStorageAdapter::new(auth.mysql_addr.clone());
        return Ok(Arc::new(driver));
    }

    Err(MqttBrokerError::UnavailableStorageType)
}

pub fn authentication_acl() -> bool {
    false
}
