// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use std::sync::Arc;

use crate::handler::cache_manager::CacheManager;

use super::Authentication;
use axum::async_trait;
use common_base::error::{common::CommonError, mqtt_broker::MQTTBrokerError};

pub struct Plaintext {
    username: String,
    password: String,
    cache_manager: Arc<CacheManager>,
}

impl Plaintext {
    pub fn new(username: String, password: String, cache_manager: Arc<CacheManager>) -> Self {
        return Plaintext {
            username,
            password,
            cache_manager,
        };
    }
}

#[async_trait]
impl Authentication for Plaintext {
    async fn apply(&self) -> Result<bool, MQTTBrokerError> {
        if let Some(user) = self.cache_manager.user_info.get(&self.username) {
            return Ok(user.password == self.password);
        }
        return Err(MQTTBrokerError::UserDoesNotExist);
    }
}

#[cfg(test)]
mod test {
    use super::Plaintext;
    use crate::{handler::cache_manager::CacheManager, security::authentication::Authentication};
    use clients::poll::ClientPool;
    use common_base::config::broker_mqtt::BrokerMQTTConfig;
    use metadata_struct::mqtt::user::MQTTUser;
    use protocol::mqtt::common::Login;
    use std::sync::Arc;

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
        let pt = Plaintext::new(login.username, login.password, cache_manager.clone());
        let res = pt.apply().await.unwrap();
        assert!(res);

        let login = Login {
            username,
            password: "pwd1111".to_string(),
        };
        let pt = Plaintext::new(login.username, login.password, cache_manager.clone());
        let res = pt.apply().await.unwrap();
        assert!(!res);
    }
}
