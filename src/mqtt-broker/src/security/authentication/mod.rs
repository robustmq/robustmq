use self::plaintext::Plaintext;
use crate::metadata::{cache::MetadataCache, cluster::Cluster};
use axum::async_trait;
use bytes::Bytes;
use common_base::errors::RobustMQError;
use protocol::mqtt::{ConnectProperties, Login};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod plaintext;

#[async_trait]
pub trait Authentication {
    async fn apply(&self) -> Result<bool, RobustMQError>;
}

pub async fn authentication_login<T>(
    metadata_cache: Arc<RwLock<MetadataCache<T>>>,
    cluster: &Cluster,
    login: Option<Login>,
    _: &Option<ConnectProperties>
) -> Result<bool, RobustMQError> {
    // Supports non-secret login
    if cluster.secret_free_login() {
        return Ok(true);
    }

    // Basic authentication mode
    if let Some(info) = login {
        let cache = metadata_cache.read().await;
        let plaintext = Plaintext::new(info, &cache.user_info);
        return plaintext.apply().await;
    }

    // Extended authentication mode

    return Ok(false);
}

