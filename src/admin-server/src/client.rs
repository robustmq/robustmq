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

use crate::{path::*, tool::PageReplyData};
use common_base::http_response::AdminServerResponse;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// HTTP client errors
#[derive(Error, Debug)]
pub enum HttpClientError {
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("JSON serialization failed: {0}")]
    JsonSerializationFailed(#[from] serde_json::Error),
    #[error("Server returned error: code={code}, message={message}")]
    ServerError { code: u64, message: String },
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

/// HTTP client for RobustMQ Admin Server API
#[derive(Clone)]
pub struct AdminHttpClient {
    client: Client,
    base_url: String,
}

impl AdminHttpClient {
    /// Create a new HTTP client instance
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.into(),
        }
    }

    /// Create a new HTTP client with custom timeout
    pub fn with_timeout(base_url: impl Into<String>, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.into(),
        }
    }

    /// Make a POST request to the admin server
    pub async fn post<T, R>(&self, endpoint: &str, request: &T) -> Result<R, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let url = self.build_url(endpoint)?;
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;
        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(HttpClientError::ServerError {
                code: status.as_u16() as u64,
                message: response_text,
            });
        }

        // Try to parse as AdminServerResponse first
        match serde_json::from_str::<AdminServerResponse<R>>(&response_text) {
            Ok(api_response) => {
                if api_response.code == 0 {
                    Ok(api_response.data)
                } else {
                    Err(HttpClientError::ServerError {
                        code: api_response.code,
                        message: format!("Server error code: {}", api_response.code),
                    })
                }
            }
            Err(_) => {
                // If not ApiResponse format, try to parse directly as the expected type
                serde_json::from_str::<R>(&response_text)
                    .map_err(HttpClientError::JsonSerializationFailed)
            }
        }
    }

    /// Make a POST request and return raw response text (for simple string responses)
    pub async fn post_raw<T>(&self, endpoint: &str, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        let url = self.build_url(endpoint)?;

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(HttpClientError::ServerError {
                code: status.as_u16() as u64,
                message: response_text,
            });
        }

        Ok(response_text)
    }

    /// Make a GET request (for the root endpoint)
    pub async fn get(&self, endpoint: &str) -> Result<String, HttpClientError> {
        let url = self.build_url(endpoint)?;

        let response = self.client.get(&url).send().await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(HttpClientError::ServerError {
                code: status.as_u16() as u64,
                message: response_text,
            });
        }

        Ok(response_text)
    }

    /// Build full URL from endpoint
    fn build_url(&self, endpoint: &str) -> Result<String, HttpClientError> {
        let endpoint = if endpoint.starts_with('/') {
            endpoint
        } else {
            &format!("/{endpoint}")
        };
        let base_url = self.base_url.trim_end_matches('/');

        let url = format!("{base_url}{endpoint}");

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(HttpClientError::InvalidUrl(format!(
                "URL must start with http:// or https://, got: {url}",
            )));
        }

        Ok(url)
    }
}

/// Convenience methods for common admin server operations
impl AdminHttpClient {
    /// Get service version information
    pub async fn get_version(&self) -> Result<String, HttpClientError> {
        self.get("/").await
    }

    /// Get cluster overview
    pub async fn get_cluster_overview<T>(&self) -> Result<T, HttpClientError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let empty_request = serde_json::json!({});
        self.post(&api_path(MQTT_OVERVIEW_PATH), &empty_request)
            .await
    }

    /// Get client list
    pub async fn get_client_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_CLIENT_LIST_PATH), request).await
    }

    /// Get session list
    pub async fn get_session_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_SESSION_LIST_PATH), request).await
    }

    /// Get topic list
    pub async fn get_topic_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_TOPIC_LIST_PATH), request).await
    }

    /// Get topic detail
    pub async fn get_topic_detail<T, R>(&self, request: &T) -> Result<R, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_TOPIC_DETAIL_PATH), request).await
    }

    /// Get subscription list
    pub async fn get_subscribe_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_SUBSCRIBE_LIST_PATH), request)
            .await
    }

    /// Get user list
    pub async fn get_user_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_USER_LIST_PATH), request).await
    }

    /// Create user
    pub async fn create_user<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_USER_CREATE_PATH), request)
            .await
    }

    /// Delete user
    pub async fn delete_user<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_USER_DELETE_PATH), request)
            .await
    }

    /// Get ACL list
    pub async fn get_acl_list<T, R>(&self, request: &T) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_ACL_LIST_PATH), request).await
    }

    /// Create ACL rule
    pub async fn create_acl<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_ACL_CREATE_PATH), request)
            .await
    }

    /// Delete ACL rule
    pub async fn delete_acl<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_ACL_DELETE_PATH), request)
            .await
    }

    /// Get blacklist
    pub async fn get_blacklist<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_BLACKLIST_LIST_PATH), request)
            .await
    }

    /// Create blacklist entry
    pub async fn create_blacklist<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_BLACKLIST_CREATE_PATH), request)
            .await
    }

    /// Delete blacklist entry
    pub async fn delete_blacklist<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_BLACKLIST_DELETE_PATH), request)
            .await
    }

    /// Get connector list
    pub async fn get_connector_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_CONNECTOR_LIST_PATH), request)
            .await
    }

    /// Create connector
    pub async fn create_connector<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_CONNECTOR_CREATE_PATH), request)
            .await
    }

    /// Delete connector
    pub async fn delete_connector<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_CONNECTOR_DELETE_PATH), request)
            .await
    }

    /// Get schema list
    pub async fn get_schema_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_SCHEMA_LIST_PATH), request).await
    }

    /// Create schema
    pub async fn create_schema<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_SCHEMA_CREATE_PATH), request)
            .await
    }

    /// Delete schema
    pub async fn delete_schema<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_SCHEMA_DELETE_PATH), request)
            .await
    }

    /// Get schema binding list
    pub async fn get_schema_bind_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_SCHEMA_BIND_LIST_PATH), request)
            .await
    }

    /// Create schema binding
    pub async fn create_schema_bind<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_SCHEMA_BIND_CREATE_PATH), request)
            .await
    }

    /// Delete schema binding
    pub async fn delete_schema_bind<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_SCHEMA_BIND_DELETE_PATH), request)
            .await
    }

    /// Get system alarm list
    pub async fn get_system_alarm_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_SYSTEM_ALARM_LIST_PATH), request)
            .await
    }

    /// Set cluster configuration
    pub async fn set_cluster_config<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(CLUSTER_CONFIG_SET_PATH), request)
            .await
    }

    /// Set cluster configuration
    pub async fn get_cluster_config<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(CLUSTER_CONFIG_GET_PATH), request)
            .await
    }

    /// Get flapping detection list
    pub async fn get_flapping_detect_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_FLAPPING_DETECT_LIST_PATH), request)
            .await
    }

    /// Get topic rewrite rules list
    pub async fn get_topic_rewrite_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_TOPIC_REWRITE_LIST_PATH), request)
            .await
    }

    /// Create topic rewrite rule
    pub async fn create_topic_rewrite<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_TOPIC_REWRITE_CREATE_PATH), request)
            .await
    }

    /// Delete topic rewrite rule
    pub async fn delete_topic_rewrite<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_TOPIC_REWRITE_DELETE_PATH), request)
            .await
    }

    /// Get auto subscribe list
    pub async fn get_auto_subscribe_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_AUTO_SUBSCRIBE_LIST_PATH), request)
            .await
    }

    /// Create auto subscribe rule
    pub async fn create_auto_subscribe<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_AUTO_SUBSCRIBE_CREATE_PATH), request)
            .await
    }

    /// Delete auto subscribe rule
    pub async fn delete_auto_subscribe<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_AUTO_SUBSCRIBE_DELETE_PATH), request)
            .await
    }

    /// Get slow subscribe list
    pub async fn get_slow_subscribe_list<T, R>(
        &self,
        request: &T,
    ) -> Result<PageReplyData<R>, HttpClientError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        self.post(&api_path(MQTT_SLOW_SUBSCRIBE_LIST_PATH), request)
            .await
    }

    /// Get subscribe detail
    pub async fn get_subscribe_detail<T>(&self, request: &T) -> Result<String, HttpClientError>
    where
        T: Serialize,
    {
        self.post_raw(&api_path(MQTT_SUBSCRIBE_DETAIL_PATH), request)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let client = AdminHttpClient::new("http://localhost:8080");

        // Test with leading slash
        assert_eq!(
            client.build_url(&api_path(MQTT_OVERVIEW_PATH)).unwrap(),
            "http://localhost:8080/api/mqtt/overview"
        );

        // Test without leading slash
        assert_eq!(
            client
                .build_url(&api_path(MQTT_OVERVIEW_PATH)[1..])
                .unwrap(),
            "http://localhost:8080/api/mqtt/overview"
        );

        // Test with trailing slash in base URL
        let client2 = AdminHttpClient::new("http://localhost:8080/");
        assert_eq!(
            client2.build_url(&api_path(MQTT_OVERVIEW_PATH)).unwrap(),
            "http://localhost:8080/api/mqtt/overview"
        );
    }

    #[test]
    fn test_invalid_url() {
        let client = AdminHttpClient::new("invalid-url");
        let result = client.build_url("/test");
        assert!(matches!(result, Err(HttpClientError::InvalidUrl(_))));
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = AdminHttpClient::new("http://localhost:8080");
        assert_eq!(client.base_url, "http://localhost:8080");

        let client_with_timeout =
            AdminHttpClient::with_timeout("http://localhost:8080", Duration::from_secs(10));
        assert_eq!(client_with_timeout.base_url, "http://localhost:8080");
    }
}
