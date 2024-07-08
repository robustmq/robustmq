use std::sync::Arc;

use crate::core::cache_manager::CacheManager;

use super::Authentication;
use axum::async_trait;
use common_base::errors::RobustMQError;
use protocol::mqtt::common::Login;

pub struct Plaintext {
    login: Login,
    cache_manager: Arc<CacheManager>,
}

impl Plaintext {
    pub fn new(login: Login, cache_manager: Arc<CacheManager>) -> Self {
        return Plaintext {
            login,
            cache_manager,
        };
    }
}

#[async_trait]
impl Authentication for Plaintext {
    async fn apply(&self) -> Result<bool, RobustMQError> {
        if let Some(user) = self.cache_manager.user_info.get(&self.login.username) {
            return Ok(user.password == self.login.password);
        }
        return Ok(false);
    }
}

#[cfg(test)]
mod test {
    use clients::poll::ClientPool;
    use common_base::config::broker_mqtt::BrokerMQTTConfig;
    use metadata_struct::mqtt::user::MQTTUser;
    use protocol::mqtt::common::Login;
    use std::sync::Arc;
    use crate::{core::cache_manager::CacheManager, security::authentication::Authentication};
    use super::Plaintext;

    #[tokio::test]
    pub async fn plaintext_test() {
        let mut conf = BrokerMQTTConfig::default();
        conf.cluster_name = "test".to_string();
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(100));
        let cache_manager: Arc<CacheManager> = Arc::new(CacheManager::new(
            client_poll.clone(),
            conf.cluster_name.clone(),
        ));
        let username = "lobo".to_string();
        let password = "pwd123".to_string();
        let user = MQTTUser {
            username: username.clone(),
            password: password.clone(),
            is_superuser: true,
        };
        cache_manager.add_user(user);

        let login = Login {
            username: username.clone(),
            password: password.clone(),
        };
        let pt = Plaintext::new(login, cache_manager.clone());
        let res = pt.apply().await.unwrap();
        assert!(res);

        let login = Login {
            username,
            password: "pwd1111".to_string(),
        };
        let pt = Plaintext::new(login, cache_manager.clone());
        let res = pt.apply().await.unwrap();
        assert!(!res);
    }
}
