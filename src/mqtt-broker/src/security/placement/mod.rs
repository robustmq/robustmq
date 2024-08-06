use axum::async_trait;
use common_base::errors::RobustMQError;
use metadata_struct::mqtt::user::MQTTUser;

use super::AuthStorageAdapter;

pub struct PlacementAuthStorageAdapter {}

impl PlacementAuthStorageAdapter {
    pub fn new() -> PlacementAuthStorageAdapter {
        return PlacementAuthStorageAdapter {};
    }
}

#[async_trait]
impl AuthStorageAdapter for PlacementAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<Vec<MQTTUser>, RobustMQError> {
        return Ok(Vec::new());
    }

    async fn get_user(&self, username: String) -> Result<Option<MQTTUser>, RobustMQError> {
        return Ok(None);
    }
}
