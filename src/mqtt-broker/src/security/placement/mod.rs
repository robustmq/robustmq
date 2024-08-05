use axum::async_trait;
use common_base::errors::RobustMQError;
use metadata_struct::mqtt::user::MQTTUser;

use super::AuthStorageAdapter;

pub struct PlacementAuthStorageAdapter {}

#[async_trait]
impl AuthStorageAdapter for PlacementAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<Vec<MQTTUser>, RobustMQError> {
        return Ok(Vec::new());
    }

    async fn create_user(&self) -> Result<(), RobustMQError> {
        return Ok(());
    }

    async fn delete_user(&self) -> Result<(), RobustMQError> {
        return Ok(());
    }

    async fn update_user(&self) -> Result<(), RobustMQError> {
        return Ok(());
    }
}
