use axum::async_trait;
use common_base::errors::RobustMQError;
use metadata_struct::mqtt::user::MQTTUser;

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

pub struct Acl {
    pub allow: AclAllow,
    pub ip_addr: String,
    pub username: String,
    pub client_id: String,
    pub access: AclAccess,
    pub topic: String,
}

pub enum AclAllow {
    Deny,
    Allow,
}

pub enum AclAccess {
    Subscribe,
    Publish,
    PubSub,
}
