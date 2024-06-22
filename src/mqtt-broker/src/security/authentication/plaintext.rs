use std::sync::Arc;

use crate::core::cache_manager::CacheManager;

use super::Authentication;
use axum::async_trait;
use common_base::errors::RobustMQError;
use protocol::mqtt::common::Login;

pub struct Plaintext {
    login: Login,
    cache_manager: Arc<CacheManager>,
}

impl Plaintext {
    pub fn new(login: Login, cache_manager: Arc<CacheManager>) -> Self {
        return Plaintext {
            login,
            cache_manager,
        };
    }
}

#[async_trait]
impl Authentication for Plaintext {
    async fn apply(&self) -> Result<bool, RobustMQError> {
        if let Some(user) = self.cache_manager.user_info.get(&self.login.username) {
            return Ok(user.password == self.login.password);
        }
        return Ok(false);
    }
}
