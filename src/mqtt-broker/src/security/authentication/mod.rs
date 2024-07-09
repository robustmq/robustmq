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
    cache_manager: &Arc<CacheManager>,
    login: &Option<Login>,
    connect_properties: &Option<ConnectProperties>,
    addr: &SocketAddr,
) -> Result<bool, RobustMQError> {
    let cluster = cache_manager.get_cluster_info();
    // Supports non-secret login
    if cluster.is_secret_free_login() {
        return Ok(true);
    }

    // Basic authentication mode
    if let Some(info) = login {
        let plaintext = Plaintext::new(info.clone(), cache_manager.clone());
        return plaintext.apply().await;
    }

    // todoExtended authentication mode

    return Ok(false);
}

pub fn is_ip_blacklist(addr: &SocketAddr) -> bool {
    return false;
}

#[cfg(test)]
mod test {
    use super::authentication_login;
    use super::is_ip_blacklist;
    use crate::core::cache_manager::CacheManager;
    use clients::poll::ClientPool;
    use common_base::config::broker_mqtt::BrokerMQTTConfig;
    use std::sync::Arc;

    #[tokio::test]
    pub async fn authentication_login_test() {
        let mut conf = BrokerMQTTConfig::default();
        conf.cluster_name = "test".to_string();

        let client_poll = Arc::new(ClientPool::new(100));
        let cache_manager = Arc::new(CacheManager::new(
            client_poll.clone(),
            conf.cluster_name.clone(),
        ));

        let login = None;
        let connect_properties = None;
        let addr = "127.0.0.1:1000".parse().unwrap();
        let res = authentication_login(&cache_manager, &login, &connect_properties, &addr)
            .await
            .unwrap();
        assert!(res);
    }

    #[tokio::test]
    pub async fn is_ip_blacklist_test() {
        assert!(!is_ip_blacklist(&"127.0.0.1:1000".parse().unwrap()))
    }
}
