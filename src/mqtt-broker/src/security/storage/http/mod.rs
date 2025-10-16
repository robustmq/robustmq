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
use crate::handler::error::MqttBrokerError;
use crate::security::AuthStorageAdapter;
use axum::async_trait;
use common_config::security::HttpConfig;
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use metadata_struct::mqtt::user::MqttUser;
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

/// HTTP auth response
#[derive(Debug, Deserialize)]
pub struct HttpAuthResponse {
    pub result: String,             // "allow", "deny", "ignore"
    pub is_superuser: Option<bool>, // is superuser
    #[serde(default)]
    pub client_attrs: Option<HashMap<String, String>>, // client attrs
    pub expire_at: Option<u64>,     // expire at
}

/// HTTP auth storage adapter
pub struct HttpAuthStorageAdapter {
    config: HttpConfig,
}

impl HttpAuthStorageAdapter {
    /// Create HTTP auth storage adapter with HttpConfig
    pub fn new(config: HttpConfig) -> Self {
        HttpAuthStorageAdapter { config }
    }

    /// Render template string, replace placeholders
    fn render_template(
        &self,
        template: &str,
        username: &str,
        password: &str,
        client_id: &str,
        source_ip: &str,
    ) -> String {
        template
            .replace("${username}", username)
            .replace("${password}", password)
            .replace("${clientid}", client_id)
            .replace("${source_ip}", source_ip)
    }

    /// Render URL
    fn render_url(
        &self,
        username: &str,
        password: &str,
        client_id: &str,
        source_ip: &str,
    ) -> String {
        self.render_template(&self.config.url, username, password, client_id, source_ip)
    }

    /// Render request headers
    fn render_headers(
        &self,
        username: &str,
        password: &str,
        client_id: &str,
        source_ip: &str,
    ) -> HashMap<String, String> {
        if let Some(ref headers) = self.config.headers {
            headers
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        self.render_template(v, username, password, client_id, source_ip),
                    )
                })
                .collect()
        } else {
            HashMap::new()
        }
    }

    /// Render request body
    fn render_body(
        &self,
        username: &str,
        password: &str,
        client_id: &str,
        source_ip: &str,
    ) -> HashMap<String, String> {
        if let Some(ref body) = self.config.body {
            body.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        self.render_template(v, username, password, client_id, source_ip),
                    )
                })
                .collect()
        } else {
            HashMap::new()
        }
    }

    /// Send HTTP auth request
    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
        client_id: &str,
        source_ip: &str,
    ) -> Result<HttpAuthResponse, MqttBrokerError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| MqttBrokerError::HttpRequestError(e.to_string()))?;

        let url = self.render_url(username, password, client_id, source_ip);
        let headers = self.render_headers(username, password, client_id, source_ip);

        let mut request_builder = match self.config.method.to_uppercase().as_str() {
            "GET" => {
                let mut query_params = Vec::new();
                let body_params = self.render_body(username, password, client_id, source_ip);
                for (key, value) in body_params {
                    query_params.push((key, value));
                }
                client.get(&url).query(&query_params)
            }
            "POST" => {
                let body_params = self.render_body(username, password, client_id, source_ip);
                client.post(&url).json(&body_params)
            }
            _ => {
                return Err(MqttBrokerError::UnsupportedHttpMethod(
                    self.config.method.clone(),
                ))
            }
        };

        // Add request headers
        for (key, value) in headers {
            request_builder = request_builder.header(&key, &value);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| MqttBrokerError::HttpRequestError(e.to_string()))?;

        // Check response status code
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

    /// Verify user auth
    pub async fn verify_user(
        &self,
        username: &str,
        password: &str,
        client_id: &str,
        source_ip: &str,
    ) -> Result<Option<MqttUser>, MqttBrokerError> {
        let response = self
            .authenticate_user(username, password, client_id, source_ip)
            .await?;

        match response.result.as_str() {
            "allow" => Ok(Some(MqttUser {
                username: username.to_string(),
                password: password.to_string(),
                salt: None,
                is_superuser: response.is_superuser.unwrap_or(false),
            })),
            "deny" => Ok(None),
            "ignore" => Err(MqttBrokerError::UserDoesNotExist),
            _ => Err(MqttBrokerError::HttpResponseParseError(format!(
                "Unknown result: {}",
                response.result
            ))),
        }
    }
}

#[async_trait]
impl AuthStorageAdapter for HttpAuthStorageAdapter {
    /// HTTP adapter does not support read all users, return empty collection
    async fn read_all_user(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        Ok(DashMap::new())
    }

    /// HTTP adapter does not support read all ACLs, return empty vector
    async fn read_all_acl(&self) -> Result<Vec<MqttAcl>, MqttBrokerError> {
        Ok(Vec::new())
    }

    /// HTTP adapter does not support read all blacklists, return empty vector
    async fn read_all_blacklist(&self) -> Result<Vec<MqttAclBlackList>, MqttBrokerError> {
        Ok(Vec::new())
    }

    /// HTTP adapter does not support directly get user, need to provide password and other authentication information
    /// This method returns None, actual authentication please use verify_user method
    async fn get_user(&self, _username: String) -> Result<Option<MqttUser>, MqttBrokerError> {
        Ok(None)
    }

    /// HTTP adapter does not support save user
    async fn save_user(&self, _user_info: MqttUser) -> ResultMqttBrokerError {
        Err(MqttBrokerError::CommonError(
            "HTTP adapter does not support save_user".to_string(),
        ))
    }

    /// HTTP adapter does not support delete user
    async fn delete_user(&self, _username: String) -> ResultMqttBrokerError {
        Err(MqttBrokerError::CommonError(
            "HTTP adapter does not support delete_user".to_string(),
        ))
    }

    /// HTTP adapter does not support save ACL
    async fn save_acl(&self, _acl: MqttAcl) -> ResultMqttBrokerError {
        Err(MqttBrokerError::CommonError(
            "HTTP adapter does not support save_acl".to_string(),
        ))
    }

    /// HTTP adapter does not support delete ACL
    async fn delete_acl(&self, _acl: MqttAcl) -> ResultMqttBrokerError {
        Err(MqttBrokerError::CommonError(
            "HTTP adapter does not support delete_acl".to_string(),
        ))
    }

    /// HTTP adapter does not support save blacklist
    async fn save_blacklist(&self, _blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        Err(MqttBrokerError::CommonError(
            "HTTP adapter does not support save_blacklist".to_string(),
        ))
    }

    /// HTTP adapter does not support delete blacklist
    async fn delete_blacklist(&self, _blacklist: MqttAclBlackList) -> ResultMqttBrokerError {
        Err(MqttBrokerError::CommonError(
            "HTTP adapter does not support delete_blacklist".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_http_template_rendering() {
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

        let http_adapter = HttpAuthStorageAdapter::new(http_config);

        let rendered_url =
            http_adapter.render_url("test_user", "test_password", "test_client", "127.0.0.1");
        assert_eq!(
            rendered_url,
            "http://127.0.0.1:8080/auth?client=test_client"
        );

        let rendered_headers =
            http_adapter.render_headers("test_user", "test_password", "test_client", "127.0.0.1");
        assert_eq!(
            rendered_headers.get("Authorization"),
            Some(&"Bearer test_user".to_string())
        );

        let rendered_body =
            http_adapter.render_body("test_user", "test_password", "test_client", "127.0.0.1");
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
