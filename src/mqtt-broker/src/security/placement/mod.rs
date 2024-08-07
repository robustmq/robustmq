use std::sync::Arc;

use axum::async_trait;
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use dashmap::DashMap;
use metadata_struct::mqtt::user::MQTTUser;

use crate::storage::user::UserStorage;

use super::AuthStorageAdapter;

pub struct PlacementAuthStorageAdapter {
    user_storage: UserStorage,
}

impl PlacementAuthStorageAdapter {
    pub fn new(client_poll: Arc<ClientPool>) -> PlacementAuthStorageAdapter {
        let user_storage = UserStorage::new(client_poll);
        return PlacementAuthStorageAdapter { user_storage };
    }
}

#[async_trait]
impl AuthStorageAdapter for PlacementAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<DashMap<String, MQTTUser>, RobustMQError> {
        return self.user_storage.user_list().await;
    }

    async fn get_user(&self, username: String) -> Result<Option<MQTTUser>, RobustMQError> {
        return self.user_storage.get_user(username).await;
    }
}
