use self::plaintext::Plaintext;
use crate::core::cache_manager::CacheManager;
use axum::async_trait;
use common_base::errors::RobustMQError;
use protocol::mqtt::common::{ConnectProperties, Login};
use std::{net::SocketAddr, sync::Arc};

pub mod plaintext;

#[async_trait]
pub trait Authentication {
    async fn apply(&self) -> Result<bool, RobustMQError>;
}

pub async fn authentication_login(
    cache_manager: Arc<CacheManager>,
    login: Option<Login>,
    connect_properties: &Option<ConnectProperties>,
    addr: SocketAddr,
) -> Result<bool, RobustMQError> {
    let cluster = cache_manager.get_cluster_info();
    // Supports non-secret login
    if cluster.is_secret_free_login() {
        return Ok(true);
    }

    // Basic authentication mode
    if let Some(info) = login {
        let plaintext = Plaintext::new(info, cache_manager.clone());
        return plaintext.apply().await;
    }

    // todoExtended authentication mode

    return Ok(false);
}

pub fn authentication_acl() -> bool {
    return true;
}

pub fn is_ip_blacklist(addr: &SocketAddr) -> bool {
    return false;
}

