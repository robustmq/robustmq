// Copyright 2023 RobustMQ Team
//
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

use super::Authentication;
use crate::handler::cache::MQTTCacheManager;
use crate::handler::error::MqttBrokerError;
use crate::security::storage::storage_trait::AuthStorageAdapter;
use axum::async_trait;
use std::sync::Arc;

pub struct Plaintext {
    username: String,
    password: String,
    cache_manager: Arc<MQTTCacheManager>,
}

impl Plaintext {
    pub fn new(username: String, password: String, cache_manager: Arc<MQTTCacheManager>) -> Self {
        Plaintext {
            username,
            password,
            cache_manager,
        }
    }
}

pub async fn plaintext_check_login(
    driver: &Arc<dyn AuthStorageAdapter + Send + 'static + Sync>,
    cache_manager: &Arc<MQTTCacheManager>,
    username: &str,
    password: &str,
) -> Result<bool, MqttBrokerError> {
    let plaintext = Plaintext::new(
        username.to_owned(),
        password.to_owned(),
        cache_manager.clone(),
    );
    match plaintext.apply().await {
        Ok(flag) => {
            if flag {
                return Ok(true);
            }
        }
        Err(e) => {
            // If the user does not exist, try to get the user information from the storage layer
            if e.to_string() == MqttBrokerError::UserDoesNotExist.to_string() {
                return try_get_check_user_by_driver(driver, cache_manager, username, password)
                    .await;
            }
            return Err(e);
        }
    }
    Ok(false)
}

async fn try_get_check_user_by_driver(
    driver: &Arc<dyn AuthStorageAdapter + Send + 'static + Sync>,
    cache_manager: &Arc<MQTTCacheManager>,
    username: &str,
    password: &str,
) -> Result<bool, MqttBrokerError> {
    if let Some(user) = driver.get_user(username.to_owned()).await? {
        cache_manager.add_user(user.clone());

        let plaintext = Plaintext::new(
            username.to_owned(),
            password.to_owned(),
            cache_manager.clone(),
        );

        if plaintext.apply().await? {
            return Ok(true);
        }
    }

    Ok(false)
}

#[async_trait]
impl Authentication for Plaintext {
    async fn apply(&self) -> Result<bool, MqttBrokerError> {
        if let Some(user) = self.cache_manager.user_info.get(&self.username) {
            return Ok(user.password == self.password);
        }
        return Err(MqttBrokerError::UserDoesNotExist);
    }
}

#[cfg(test)]
mod test {
    use common_base::tools::now_second;
    use metadata_struct::mqtt::user::MqttUser;
    use protocol::mqtt::common::Login;
    use std::sync::Arc;

    use super::Plaintext;
    use crate::common::tool::test_build_mqtt_cache_manager;
    use crate::handler::cache::MQTTCacheManager;
    use crate::security::login::Authentication;

    #[tokio::test]
    pub async fn plaintext_test() {
        let cache_manager: Arc<MQTTCacheManager> = test_build_mqtt_cache_manager().await;
        let username = "lobo".to_string();
        let password = "pwd123".to_string();
        let user = MqttUser {
            username: username.clone(),
            password: password.clone(),
            salt: None,
            is_superuser: true,
            create_time: now_second(),
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
