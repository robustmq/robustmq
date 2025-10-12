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
use common_config::security::HttpConfig;
use metadata_struct::mqtt::user::MqttUser;
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub struct HttpAuth {
    username: String,
    password: String,
    client_id: String,
    source_ip: String,
    http_config: HttpConfig,
    cache_manager: Arc<MQTTCacheManager>,
}

#[derive(Debug, Deserialize)]
pub struct HttpAuthResponse {
    pub result: String,             // "allow", "deny", "ignore"
    pub is_superuser: Option<bool>, // is superuser
    #[serde(default)]
    pub client_attrs: Option<HashMap<String, String>>, // client attrs
    pub expire_at: Option<u64>,     // expire at
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
            http_config,
            cache_manager,
        }
    }

    /// render template string, replace placeholder
    fn render_template(&self, template: &str) -> String {
        template
            .replace("${username}", &self.username)
            .replace("${password}", &self.password)
            .replace("${clientid}", &self.client_id)
            .replace("${source_ip}", &self.source_ip)
    }

    /// render URL
    fn render_url(&self) -> String {
        self.render_template(&self.http_config.url)
    }

    /// render request headers
    fn render_headers(&self) -> HashMap<String, String> {
        if let Some(ref headers) = self.http_config.headers {
            headers
                .iter()
                .map(|(k, v)| (k.clone(), self.render_template(v)))
                .collect()
        } else {
            HashMap::new()
        }
    }

    /// render request body
    fn render_body(&self) -> HashMap<String, String> {
        if let Some(ref body) = self.http_config.body {
            body.iter()
                .map(|(k, v)| (k.clone(), self.render_template(v)))
                .collect()
        } else {
            HashMap::new()
        }
    }

    /// send HTTP authentication request
    async fn authenticate(&self) -> Result<HttpAuthResponse, MqttBrokerError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| MqttBrokerError::HttpRequestError(e.to_string()))?;

        let url = self.render_url();
        let headers = self.render_headers();

        let mut request_builder = match self.http_config.method.to_uppercase().as_str() {
            "GET" => {
                let mut query_params = Vec::new();
                let body_params = self.render_body();
                for (key, value) in body_params {
                    query_params.push((key, value));
                }
                client.get(&url).query(&query_params)
            }
            "POST" => {
                let body_params = self.render_body();
                client.post(&url).json(&body_params)
            }
            _ => {
                return Err(MqttBrokerError::UnsupportedHttpMethod(
                    self.http_config.method.clone(),
                ))
            }
        };

        // add request headers
        for (key, value) in headers {
            request_builder = request_builder.header(&key, &value);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| MqttBrokerError::HttpRequestError(e.to_string()))?;

        // check response status code
        if !response.status().is_success() {
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                // 4xx/5xx status code return ignore
                return Ok(HttpAuthResponse {
                    result: "ignore".to_string(),
                    is_superuser: None,
                    client_attrs: None,
                    expire_at: None,
                });
            }
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| MqttBrokerError::HttpResponseParseError(e.to_string()))?;

        serde_json::from_str::<HttpAuthResponse>(&response_text)
            .map_err(|e| MqttBrokerError::HttpResponseParseError(e.to_string()))
    }
}

/// HTTP authentication check entry function
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
            // if user does not exist, try to get user information from storage layer
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

/// try to get user from storage driver and verify
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
        // send HTTP authentication request
        let response = self.authenticate().await?;

        match response.result.as_str() {
            "allow" => {
                // update user information to cache
                let user = MqttUser {
                    username: self.username.clone(),
                    password: self.password.clone(),
                    salt: None,
                    is_superuser: response.is_superuser.unwrap_or(false),
                };
                self.cache_manager.add_user(user);
                Ok(true)
            }
            "deny" => Ok(false),
            "ignore" => Err(MqttBrokerError::UserDoesNotExist),
            _ => Err(MqttBrokerError::HttpResponseParseError(format!(
                "Unknown result: {}",
                response.result
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::tool::test_build_mqtt_cache_manager;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_http_template_rendering() {
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

        let rendered_url = http_auth.render_url();
        assert_eq!(
            rendered_url,
            "http://127.0.0.1:8080/auth?client=test_client"
        );

        let rendered_headers = http_auth.render_headers();
        assert_eq!(
            rendered_headers.get("Authorization"),
            Some(&"Bearer test_user".to_string())
        );

        let rendered_body = http_auth.render_body();
        assert_eq!(
            rendered_body.get("username"),
            Some(&"test_user".to_string())
        );
        assert_eq!(
            rendered_body.get("password"),
            Some(&"test_password".to_string())
        );
        assert_eq!(
            rendered_body.get("clientid"),
            Some(&"test_client".to_string())
        );
    }

    #[test]
    fn test_http_auth_response_parsing() {
        let json_response = r#"
        {
            "result": "allow",
            "is_superuser": true
        }
        "#;

        let response: HttpAuthResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(response.result, "allow");
        assert_eq!(response.is_superuser, Some(true));
    }

    #[test]
    fn test_http_auth_response_parsing_with_client_attrs() {
        let json_response = r#"
        {
            "result": "allow",
            "is_superuser": false,
            "client_attrs": {
                "role": "admin",
                "sn": "10c61f1a1f47"
            },
            "expire_at": 1654254601
        }
        "#;

        let response: HttpAuthResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(response.result, "allow");
        assert_eq!(response.is_superuser, Some(false));
        assert_eq!(response.expire_at, Some(1654254601));

        let client_attrs = response.client_attrs.unwrap();
        assert_eq!(client_attrs.get("role"), Some(&"admin".to_string()));
        assert_eq!(client_attrs.get("sn"), Some(&"10c61f1a1f47".to_string()));
    }

    #[test]
    fn test_http_auth_response_deny() {
        let json_response = r#"
        {
            "result": "deny"
        }
        "#;

        let response: HttpAuthResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(response.result, "deny");
        assert_eq!(response.is_superuser, None);
    }

    #[test]
    fn test_http_auth_response_ignore() {
        let json_response = r#"
        {
            "result": "ignore"
        }
        "#;

        let response: HttpAuthResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(response.result, "ignore");
        assert_eq!(response.is_superuser, None);
    }
}
