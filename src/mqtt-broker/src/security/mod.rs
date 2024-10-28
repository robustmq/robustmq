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

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use acl::is_allow_acl;
use axum::async_trait;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::config::common::Auth;
use common_base::error::common::CommonError;
use common_base::error::mqtt_broker::MqttBrokerError;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use login::plaintext::Plaintext;
use login::Authentication;
use metadata_struct::acl::mqtt_acl::{MqttAcl, MqttAclAction};
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::user::MqttUser;
use mysql::MySQLAuthStorageAdapter;
use placement::PlacementAuthStorageAdapter;
use protocol::mqtt::common::{ConnectProperties, Login, QoS, Subscribe};
use storage_adapter::StorageType;

use crate::handler::cache::CacheManager;
use crate::handler::connection::Connection;
use crate::subscribe::sub_common::get_sub_topic_id_list;

pub mod acl;
pub mod login;
pub mod mysql;
pub mod placement;
pub mod redis;

#[async_trait]
pub trait AuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, CommonError>;

    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, CommonError>;

    async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, CommonError>;

    async fn get_user(&self, username: String) -> Result<Option<MqttUser>, CommonError>;
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

    pub fn update_driver(&mut self, auth: Auth) -> Result<(), CommonError> {
        let driver = match build_driver(self.client_pool.clone(), auth) {
            Ok(driver) => driver,
            Err(e) => {
                return Err(e);
            }
        };
        self.driver = driver;
        Ok(())
    }

    pub async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, CommonError> {
        self.driver.read_all_user().await
    }

    pub async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, CommonError> {
        self.driver.read_all_acl().await
    }

    pub async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, CommonError> {
        self.driver.read_all_blacklist().await
    }

    pub async fn check_login_auth(
        &self,
        login: &Option<Login>,
        _: &Option<ConnectProperties>,
        _: &SocketAddr,
    ) -> Result<bool, CommonError> {
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

    pub async fn allow_publish(
        &self,
        connection: &Connection,
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

    pub async fn allow_subscribe(&self, connection: &Connection, subscribe: &Subscribe) -> bool {
        for filter in subscribe.filters.clone() {
            let topic_list = get_sub_topic_id_list(self.cache_manager.clone(), filter.path).await;
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
    ) -> Result<bool, CommonError> {
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
                return Err(e.into());
            }
        }

        Ok(false)
    }

    async fn try_get_check_user_by_driver(&self, username: &str) -> Result<bool, CommonError> {
        match self.driver.get_user(username.to_owned()).await {
            Ok(Some(user)) => {
                self.cache_manager.add_user(user.clone());
                let plaintext = Plaintext::new(
                    user.username.clone(),
                    user.password.clone(),
                    self.cache_manager.clone(),
                );
                match plaintext.apply().await {
                    Ok(flag) => {
                        if flag {
                            return Ok(true);
                        }
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
            Ok(None) => {
                return Ok(false);
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(false)
    }
}

pub fn build_driver(
    client_pool: Arc<ClientPool>,
    auth: Auth,
) -> Result<Arc<dyn AuthStorageAdapter + Send + 'static + Sync>, CommonError> {
    let storage_type = StorageType::from_str(&auth.storage_type)
        .map_err(|_| CommonError::UnavailableStorageType)?;
    if matches!(storage_type, StorageType::Placement) {
        let driver = PlacementAuthStorageAdapter::new(client_pool);
        return Ok(Arc::new(driver));
    }

    if matches!(storage_type, StorageType::Mysql) {
        let driver = MySQLAuthStorageAdapter::new(auth.mysql_addr.clone());
        return Ok(Arc::new(driver));
    }

    Err(CommonError::UnavailableStorageType)
}

pub fn authentication_acl() -> bool {
    false
}
