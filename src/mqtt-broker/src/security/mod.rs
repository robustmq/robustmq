use std::sync::Arc;

use axum::async_trait;
use common_base::{config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError};
use metadata_struct::mqtt::user::MQTTUser;
use placement::PlacementAuthStorageAdapter;

use crate::handler::cache_manager::CacheManager;

pub mod acl;
pub mod authentication;
pub mod mysql;
pub mod placement;

#[async_trait]
pub trait AuthStorageAdapter {
    async fn read_all_user(&self) -> Result<Vec<MQTTUser>, RobustMQError>;

    async fn create_user(&self) -> Result<(), RobustMQError>;

    async fn delete_user(&self) -> Result<(), RobustMQError>;

    async fn update_user(&self) -> Result<(), RobustMQError>;
}

pub struct AuthDriver {
    cache_manager: Arc<CacheManager>,
    driver: Arc<Box<dyn AuthStorageAdapter>>,
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

    pub fn plaintext_check_login(
        &self,
        username: String,
        password: String,
    ) -> Result<bool, RobustMQError> {
        let cluster = self.cache_manager.get_cluster_info();

        if cluster.is_secret_free_login() {
            return Ok(true);
        }

        

        return Ok(false);
    }
}

pub fn build_driver() -> Result<Box<dyn AuthStorageAdapter>, RobustMQError> {
    let conf = broker_mqtt_conf();
    let driver = Box::new(PlacementAuthStorageAdapter::new());
    return Ok(driver);
}
