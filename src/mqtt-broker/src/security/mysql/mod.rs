use super::AuthStorageAdapter;
use axum::async_trait;
use common_base::errors::RobustMQError;
use metadata_struct::mqtt::user::MQTTUser;

mod schema;
pub struct MySQLAuthStorageAdapter {}

#[async_trait]
impl AuthStorageAdapter for MySQLAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<Vec<MQTTUser>, RobustMQError> {
        return Ok(Vec::new());
    }

    async fn get_user(&self, username: String) -> Result<Option<MQTTUser>, RobustMQError> {
        return Ok(None);
    }
}
