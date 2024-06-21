use super::Authentication;
use metadata_struct::mqtt::user::MQTTUser;
use axum::async_trait;
use common_base::errors::RobustMQError;
use dashmap::DashMap;
use protocol::mqtt::common::Login;

pub struct Plaintext<'a> {
    login: Login,
    user_info: &'a DashMap<String, MQTTUser>,
}

impl<'a> Plaintext<'a> {
    pub fn new(login: Login, user_info: &'a DashMap<String, MQTTUser>) -> Self {
        return Plaintext { login, user_info };
    }
}

#[async_trait]
impl<'a> Authentication for Plaintext<'a> {
    async fn apply(&self) -> Result<bool, RobustMQError> {
        if let Some(user) = self.user_info.get(&self.login.username) {
            return Ok(user.password == self.login.password);
        }
        return Ok(false);
    }
}
