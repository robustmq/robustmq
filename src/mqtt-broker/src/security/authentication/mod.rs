use self::plaintext::Plaintext;
use crate::metadata::{cache::MetadataCache, cluster::Cluster};
use axum::async_trait;
use common_base::errors::RobustMQError;
use protocol::mqtt::{ConnectProperties, Login};
use std::sync::Arc;

pub mod plaintext;

#[async_trait]
pub trait Authentication {
    async fn apply(&self) -> Result<bool, RobustMQError>;
}

pub async fn authentication_login<T>(
    metadata_cache: Arc<MetadataCache<T>>,
    cluster: &Cluster,
    login: Option<Login>,
    _: &Option<ConnectProperties>,
) -> Result<bool, RobustMQError> {
    // Supports non-secret login
    if cluster.secret_free_login() {
        return Ok(true);
    }

    // Basic authentication mode
    if let Some(info) = login {
        let plaintext = Plaintext::new(info, &metadata_cache.user_info);
        return plaintext.apply().await;
    }

    // Extended authentication mode

    return Ok(false);
}
