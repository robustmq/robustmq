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

use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::MQTTCacheManager;
use crate::storage::user::UserStorage;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use std::sync::Arc;

pub async fn init_system_user(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
) -> ResultMqttBrokerError {
    let conf = broker_config();
    let system_user_info = MqttUser {
        username: conf.mqtt_runtime.default_user.clone(),
        password: conf.mqtt_runtime.default_password.clone(),
        salt: None,
        is_superuser: true,
    };
    let user_storage = UserStorage::new(client_pool.clone());
    let res = user_storage
        .get_user(system_user_info.username.clone())
        .await?;
    if res.is_some() {
        return Ok(());
    }
    user_storage.save_user(system_user_info.clone()).await?;
    cache_manager.add_user(system_user_info);
    Ok(())
}

pub fn is_super_user(cache_manager: &Arc<MQTTCacheManager>, username: &str) -> bool {
    if username.is_empty() {
        return false;
    }
    if let Some(user) = cache_manager.user_info.get(username) {
        return user.is_superuser;
    }
    false
}

#[cfg(test)]
mod test {
    use super::is_super_user;
    use crate::common::tool::test_build_mqtt_cache_manager;
    use metadata_struct::mqtt::user::MqttUser;

    #[tokio::test]
    pub async fn check_super_user_test() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            salt: None,
            is_superuser: true,
        };
        cache_manager.add_user(user.clone());

        let login_username = "".to_string();
        assert!(!is_super_user(&cache_manager, &login_username));

        let login_username = "test".to_string();
        assert!(!is_super_user(&cache_manager, &login_username));

        assert!(is_super_user(&cache_manager, &user.username));

        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            salt: None,
            is_superuser: false,
        };
        cache_manager.add_user(user.clone());
        assert!(!is_super_user(&cache_manager, &user.username));
    }
}
