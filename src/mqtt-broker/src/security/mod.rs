use crate::handler::cache_manager::CacheManager;
use authentication::{plaintext::Plaintext, Authentication};
use axum::async_trait;
use common_base::{config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError};
use metadata_struct::mqtt::user::MQTTUser;
use placement::PlacementAuthStorageAdapter;
use protocol::mqtt::common::{ConnectProperties, Login};
use std::{net::SocketAddr, sync::Arc};

pub mod acl;
pub mod authentication;
pub mod mysql;
pub mod placement;

#[async_trait]
pub trait AuthStorageAdapter {
    async fn read_all_user(&self) -> Result<Vec<MQTTUser>, RobustMQError>;

    async fn get_user(&self, username: String) -> Result<Option<MQTTUser>, RobustMQError>;
}

pub struct AuthDriver {
    cache_manager: Arc<CacheManager>,
    driver: Arc<Box<dyn AuthStorageAdapter + 'static + Sync>>,
}

impl AuthDriver {
    pub fn new(cache_manager: Arc<CacheManager>) -> AuthDriver {
        let driver = match build_driver() {
            Ok(driver) => driver,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        return AuthDriver {
            cache_manager,
            driver: Arc::new(driver),
        };
    }

    pub fn update_driver(&mut self) -> Result<(), RobustMQError> {
        let driver = match build_driver() {
            Ok(driver) => driver,
            Err(e) => {
                return Err(e);
            }
        };
        self.driver = Arc::new(driver);
        return Ok(());
    }

    pub async fn check_login(
        &self,
        login: &Option<Login>,
        connect_properties: &Option<ConnectProperties>,
        addr: &SocketAddr,
    ) -> Result<bool, RobustMQError> {
        let cluster = self.cache_manager.get_cluster_info();

        if cluster.is_secret_free_login() {
            return Ok(true);
        }
        if let Some(info) = login {
            return self
                .plaintext_check_login(&info.username, &info.password)
                .await;
        }
        return Ok(false);
    }

    async fn plaintext_check_login(
        &self,
        username: &String,
        password: &String,
    ) -> Result<bool, RobustMQError> {
        let plaintext = Plaintext::new(
            username.clone(),
            password.clone(),
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
                if e.to_string() == RobustMQError::UserDoesNotExist.to_string() {
                    return self.try_get_check_user_by_driver(username).await;
                }
                return Err(e);
            }
        }

        return Ok(false);
    }

    async fn try_get_check_user_by_driver(&self, username: &String) -> Result<bool, RobustMQError> {
        match self.driver.get_user(username.clone()).await {
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
                        return Err(e);
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
        return Ok(false);
    }
}

pub fn build_driver() -> Result<Box<dyn AuthStorageAdapter>, RobustMQError> {
    let conf = broker_mqtt_conf();
    let driver = Box::new(PlacementAuthStorageAdapter::new());
    return Ok(driver);
}
