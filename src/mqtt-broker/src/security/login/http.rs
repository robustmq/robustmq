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
use crate::core::cache::MQTTCacheManager;
use crate::core::error::MqttBrokerError;
use crate::security::storage::http::HttpAuthStorageAdapter;
use crate::security::storage::storage_trait::AuthStorageAdapter;
use axum::async_trait;
use common_config::security::HttpConfig;
use std::sync::Arc;

pub struct HttpAuth {
    username: String,
    password: String,
    client_id: String,
    source_ip: String,
    http_adapter: HttpAuthStorageAdapter,
    cache_manager: Arc<MQTTCacheManager>,
}

impl HttpAuth {
    pub fn new(
        username: String,
        password: String,
        client_id: String,
        source_ip: String,
        http_config: HttpConfig,
        cache_manager: Arc<MQTTCacheManager>,
    ) -> Self {
        HttpAuth {
            username,
            password,
            client_id,
            source_ip,
            http_adapter: HttpAuthStorageAdapter::new(http_config),
            cache_manager,
        }
    }
}

/// HTTP auth check entry function
pub async fn http_check_login(
    driver: &Arc<dyn AuthStorageAdapter + Send + 'static + Sync>,
    cache_manager: &Arc<MQTTCacheManager>,
    http_config: &HttpConfig,
    username: &str,
    password: &str,
    client_id: &str,
    source_ip: &str,
) -> Result<bool, MqttBrokerError> {
    let http_auth = HttpAuth::new(
        username.to_owned(),
        password.to_owned(),
        client_id.to_owned(),
        source_ip.to_owned(),
        http_config.clone(),
        cache_manager.clone(),
    );

    match http_auth.apply().await {
        Ok(flag) => {
            if flag {
                return Ok(true);
            }
        }
        Err(e) => {
            // If user does not exist, try to get user information from storage layer
            if e.to_string() == MqttBrokerError::UserDoesNotExist.to_string() {
                return try_get_check_user_by_driver(
                    driver,
                    cache_manager,
                    http_config,
                    username,
                    password,
                    client_id,
                    source_ip,
                )
                .await;
            }
            return Err(e);
        }
    }
    Ok(false)
}

/// Try to get user from storage driver and verify
async fn try_get_check_user_by_driver(
    driver: &Arc<dyn AuthStorageAdapter + Send + 'static + Sync>,
    cache_manager: &Arc<MQTTCacheManager>,
    http_config: &HttpConfig,
    username: &str,
    password: &str,
    client_id: &str,
    source_ip: &str,
) -> Result<bool, MqttBrokerError> {
    if let Some(user) = driver.get_user(username.to_owned()).await? {
        cache_manager.add_user(user.clone());

        let http_auth = HttpAuth::new(
            username.to_owned(),
            password.to_owned(),
            client_id.to_owned(),
            source_ip.to_owned(),
            http_config.clone(),
            cache_manager.clone(),
        );

        if http_auth.apply().await? {
            return Ok(true);
        }
    }

    Ok(false)
}

#[async_trait]
impl Authentication for HttpAuth {
    async fn apply(&self) -> Result<bool, MqttBrokerError> {
        // Use HTTP storage adapter for auth
        match self
            .http_adapter
            .verify_user(
                &self.username,
                &self.password,
                &self.client_id,
                &self.source_ip,
            )
            .await
        {
            Ok(Some(user)) => {
                // Update user information to cache
                self.cache_manager.add_user(user);
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tool::test_build_mqtt_cache_manager;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_http_auth_integration() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Bearer ${username}".to_string(),
        );

        let mut body = HashMap::new();
        body.insert("username".to_string(), "${username}".to_string());
        body.insert("password".to_string(), "${password}".to_string());
        body.insert("clientid".to_string(), "${clientid}".to_string());

        let http_config = HttpConfig {
            url: "http://127.0.0.1:8080/auth?client=${clientid}".to_string(),
            method: "POST".to_string(),
            headers: Some(headers),
            body: Some(body),
        };

        let http_auth = HttpAuth::new(
            "test_user".to_string(),
            "test_password".to_string(),
            "test_client".to_string(),
            "127.0.0.1".to_string(),
            http_config,
            cache_manager,
        );

        // Note: this test needs actual HTTP server, so it should be ignored in actual tests
        // Here just show how to use the new structure
        assert_eq!(http_auth.username, "test_user");
        assert_eq!(http_auth.password, "test_password");
        assert_eq!(http_auth.client_id, "test_client");
        assert_eq!(http_auth.source_ip, "127.0.0.1");
    }
}
