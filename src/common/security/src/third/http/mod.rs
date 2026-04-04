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

use async_trait::async_trait;
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use metadata_struct::auth::acl::{
    EnumAclAction, EnumAclPermission, EnumAclResourceType, SecurityAcl,
};
use metadata_struct::auth::blacklist::{get_blacklist_type_by_str, SecurityBlackList};
use metadata_struct::auth::user::SecurityUser;
use metadata_struct::mqtt::auth::storage::HttpConfig;
use metadata_struct::tenant::DEFAULT_TENANT;
use reqwest::Client;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tracing::warn;

use crate::third::storage_trait::AuthStorageAdapter;

/// HTTP auth storage adapter
pub struct HttpAuthStorageAdapter {
    config: HttpConfig,
    client: Client,
}

impl HttpAuthStorageAdapter {
    const RESOURCE_USER: &'static str = "user";
    const RESOURCE_ACL: &'static str = "acl";
    const RESOURCE_BLACKLIST: &'static str = "blacklist";

    pub fn new(config: HttpConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());
        HttpAuthStorageAdapter { config, client }
    }

    fn endpoint_for(&self, query: &str) -> String {
        if query.trim().is_empty() {
            return self.config.url.clone();
        }
        if query.starts_with("http://") || query.starts_with("https://") {
            return query.to_string();
        }
        let base = self.config.url.trim_end_matches('/');
        let path = query.trim_start_matches('/');
        format!("{base}/{path}")
    }

    fn render_template(&self, template: &str, resource: &str) -> String {
        template.replace("${resource}", resource)
    }

    fn build_headers(&self, resource: &str) -> HashMap<String, String> {
        if let Some(ref headers) = self.config.headers {
            headers
                .iter()
                .map(|(k, v)| (k.clone(), self.render_template(v, resource)))
                .collect()
        } else {
            HashMap::new()
        }
    }

    fn build_body(&self, resource: &str) -> HashMap<String, String> {
        if let Some(ref body) = self.config.body {
            body.iter()
                .map(|(k, v)| (k.clone(), self.render_template(v, resource)))
                .collect()
        } else {
            HashMap::new()
        }
    }

    async fn request_json(&self, endpoint: &str, resource: &str) -> Result<Value, CommonError> {
        let headers = self.build_headers(resource);
        let body = self.build_body(resource);
        let mut request_builder = match self.config.method.to_uppercase().as_str() {
            "GET" => self.client.get(endpoint).query(&body),
            "POST" => self.client.post(endpoint).json(&body),
            _ => {
                return Err(CommonError::UnsupportedHttpMethod(
                    self.config.method.clone(),
                ))
            }
        };

        for (key, value) in headers {
            request_builder = request_builder.header(&key, &value);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| CommonError::HttpRequestError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CommonError::HttpRequestError(format!(
                "HTTP data source request failed, status={}, endpoint={}",
                response.status(),
                endpoint
            )));
        }

        response
            .json::<Value>()
            .await
            .map_err(|e| CommonError::HttpResponseParseError(e.to_string()))
    }

    async fn fetch_resource(&self, resource: &str, query: &str) -> Result<Vec<Value>, CommonError> {
        let endpoint = self.endpoint_for(query);
        let payload = self.request_json(&endpoint, resource).await?;
        Self::extract_items(payload, resource)
    }

    fn extract_items(payload: Value, resource: &str) -> Result<Vec<Value>, CommonError> {
        match payload {
            Value::Array(items) => Ok(items),
            Value::Object(map) => {
                let keys = match resource {
                    Self::RESOURCE_USER => vec!["users", "user", "data", "items", "list"],
                    Self::RESOURCE_ACL => vec!["acls", "acl", "data", "items", "list"],
                    Self::RESOURCE_BLACKLIST => {
                        vec!["blacklists", "blacklist", "data", "items", "list"]
                    }
                    _ => vec!["data", "items", "list"],
                };
                for key in keys {
                    if let Some(value) = map.get(key) {
                        return match value {
                            Value::Array(items) => Ok(items.clone()),
                            Value::Object(_) => Ok(vec![value.clone()]),
                            _ => Err(CommonError::HttpResponseParseError(format!(
                                "field `{}` is not array/object",
                                key
                            ))),
                        };
                    }
                }
                Err(CommonError::HttpResponseParseError(
                    "unable to locate resource data in response".to_string(),
                ))
            }
            _ => Err(CommonError::HttpResponseParseError(
                "invalid response json, expected object/array".to_string(),
            )),
        }
    }

    fn as_object(value: Value) -> Option<Map<String, Value>> {
        match value {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }

    fn parse_created_to_seconds(value: Option<&Value>) -> u64 {
        match value {
            Some(Value::Number(num)) => num.as_u64().unwrap_or_else(now_second),
            Some(Value::String(s)) => {
                if let Ok(ts) = s.parse::<u64>() {
                    return ts;
                }
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                    return dt.and_utc().timestamp().max(0) as u64;
                }
                now_second()
            }
            _ => now_second(),
        }
    }

    fn parse_bool_like(value: Option<&Value>) -> bool {
        match value {
            Some(Value::Bool(v)) => *v,
            Some(Value::Number(v)) => v.as_i64().unwrap_or_default() == 1,
            Some(Value::String(v)) => v == "1" || v.eq_ignore_ascii_case("true"),
            _ => false,
        }
    }

    fn parse_string(value: Option<&Value>) -> Option<String> {
        match value {
            Some(Value::String(v)) => Some(v.clone()),
            Some(Value::Number(v)) => Some(v.to_string()),
            Some(Value::Bool(v)) => Some(v.to_string()),
            _ => None,
        }
    }

    fn parse_permission(value: Option<&Value>) -> Result<EnumAclPermission, CommonError> {
        match value {
            Some(Value::Number(v)) if v.as_i64() == Some(0) => Ok(EnumAclPermission::Deny),
            Some(Value::Number(v)) if v.as_i64() == Some(1) => Ok(EnumAclPermission::Allow),
            Some(Value::String(v)) => {
                EnumAclPermission::from_str(v).map_err(|_| CommonError::InvalidAclPermission)
            }
            _ => Err(CommonError::InvalidAclPermission),
        }
    }

    fn parse_action(value: Option<&Value>) -> Result<EnumAclAction, CommonError> {
        match value {
            Some(Value::Number(v)) if v.as_i64() == Some(0) => Ok(EnumAclAction::All),
            Some(Value::Number(v)) if v.as_i64() == Some(1) => Ok(EnumAclAction::Subscribe),
            Some(Value::Number(v)) if v.as_i64() == Some(2) => Ok(EnumAclAction::Publish),
            Some(Value::Number(v)) if v.as_i64() == Some(3) => Ok(EnumAclAction::PubSub),
            Some(Value::Number(v)) if v.as_i64() == Some(4) => Ok(EnumAclAction::Retain),
            Some(Value::Number(v)) if v.as_i64() == Some(5) => Ok(EnumAclAction::Qos),
            Some(Value::String(v)) => {
                let normalized = match v.to_ascii_lowercase().as_str() {
                    "all" => "All",
                    "subscribe" => "Subscribe",
                    "publish" => "Publish",
                    "pubsub" | "publish_subscribe" => "PubSub",
                    "retain" => "Retain",
                    "qos" => "Qos",
                    _ => v,
                };
                EnumAclAction::from_str(normalized).map_err(|_| CommonError::InvalidAclAction)
            }
            _ => Err(CommonError::InvalidAclAction),
        }
    }
}

#[async_trait]
impl AuthStorageAdapter for HttpAuthStorageAdapter {
    async fn read_all_user(&self) -> Result<Vec<SecurityUser>, CommonError> {
        let mut users = Vec::new();
        let values = self
            .fetch_resource(Self::RESOURCE_USER, &self.config.query_user)
            .await?;

        for value in values {
            let Some(map) = Self::as_object(value) else {
                warn!("HTTP user item is not object, skip");
                continue;
            };
            let Some(username) = Self::parse_string(map.get("username")) else {
                warn!("HTTP user item missing username, skip");
                continue;
            };
            let Some(password) = Self::parse_string(map.get("password")) else {
                warn!(username = %username, "HTTP user item missing password, skip");
                continue;
            };
            users.push(SecurityUser {
                tenant: DEFAULT_TENANT.to_string(),
                username,
                password,
                salt: Self::parse_string(map.get("salt")),
                is_superuser: Self::parse_bool_like(map.get("is_superuser")),
                create_time: Self::parse_created_to_seconds(map.get("created")),
            });
        }
        Ok(users)
    }

    async fn read_all_acl(&self) -> Result<Vec<SecurityAcl>, CommonError> {
        let mut acls = Vec::new();
        let values = self
            .fetch_resource(Self::RESOURCE_ACL, &self.config.query_acl)
            .await?;

        for value in values {
            let Some(map) = Self::as_object(value) else {
                warn!("HTTP acl item is not object, skip");
                continue;
            };

            let permission = match Self::parse_permission(map.get("permission")) {
                Ok(v) => v,
                Err(_) => {
                    warn!("HTTP acl item invalid permission, skip");
                    continue;
                }
            };
            let action = match Self::parse_action(map.get("access").or_else(|| map.get("action"))) {
                Ok(v) => v,
                Err(_) => {
                    warn!("HTTP acl item invalid access/action, skip");
                    continue;
                }
            };

            let username = Self::parse_string(map.get("username")).unwrap_or_default();
            let clientid = Self::parse_string(map.get("clientid")).unwrap_or_default();
            let ip = Self::parse_string(map.get("ipaddr"))
                .or_else(|| Self::parse_string(map.get("ipaddress")))
                .unwrap_or_default();
            let topics = match map.get("topics") {
                Some(Value::Array(items)) => items
                    .iter()
                    .filter_map(|item| Self::parse_string(Some(item)))
                    .collect::<Vec<_>>(),
                _ => {
                    let topic = Self::parse_string(map.get("topic")).unwrap_or_default();
                    if topic.is_empty() {
                        Vec::new()
                    } else {
                        vec![topic]
                    }
                }
            };
            if topics.is_empty() {
                warn!("HTTP acl item missing topic/topics, skip");
                continue;
            }

            let resource_type = if username.is_empty() {
                EnumAclResourceType::ClientId
            } else {
                EnumAclResourceType::User
            };
            let resource_name = if username.is_empty() {
                clientid
            } else {
                username
            };

            let name = Self::parse_string(map.get("name")).unwrap_or_default();
            let desc = Self::parse_string(map.get("desc")).unwrap_or_default();

            for topic in topics {
                acls.push(SecurityAcl {
                    name: name.clone(),
                    desc: desc.clone(),
                    tenant: DEFAULT_TENANT.to_string(),
                    permission,
                    resource_type,
                    resource_name: resource_name.clone(),
                    topic,
                    ip: ip.clone(),
                    action,
                });
            }
        }
        Ok(acls)
    }

    async fn read_all_blacklist(&self) -> Result<Vec<SecurityBlackList>, CommonError> {
        let mut blacklists = Vec::new();
        let values = self
            .fetch_resource(Self::RESOURCE_BLACKLIST, &self.config.query_blacklist)
            .await?;

        for value in values {
            let Some(map) = Self::as_object(value) else {
                warn!("HTTP blacklist item is not object, skip");
                continue;
            };
            let Some(blacklist_type_raw) = Self::parse_string(map.get("blacklist_type")) else {
                warn!("HTTP blacklist item missing blacklist_type, skip");
                continue;
            };
            let Some(resource_name) = Self::parse_string(map.get("resource_name")) else {
                warn!("HTTP blacklist item missing resource_name, skip");
                continue;
            };
            let end_time = Self::parse_string(map.get("end_time"))
                .and_then(|s| s.parse::<u64>().ok())
                .or_else(|| map.get("end_time").and_then(|v| v.as_u64()))
                .unwrap_or(0);
            let desc = Self::parse_string(map.get("desc")).unwrap_or_default();

            let normalized = match blacklist_type_raw.to_ascii_lowercase().as_str() {
                "clientid" | "client_id" => "ClientId".to_string(),
                "user" | "username" => "User".to_string(),
                "ip" | "ipaddr" | "ipaddress" => "Ip".to_string(),
                "clientidmatch" | "client_id_match" => "ClientIdMatch".to_string(),
                "usermatch" | "user_match" => "UserMatch".to_string(),
                "ipcidr" | "ip_cidr" => "IPCIDR".to_string(),
                _ => blacklist_type_raw,
            };

            let blacklist_type = match get_blacklist_type_by_str(&normalized) {
                Ok(v) => v,
                Err(_) => {
                    warn!("HTTP blacklist item invalid blacklist_type, skip");
                    continue;
                }
            };

            let name = Self::parse_string(map.get("name")).unwrap_or_default();
            blacklists.push(SecurityBlackList {
                name,
                tenant: DEFAULT_TENANT.to_string(),
                blacklist_type,
                resource_name,
                end_time,
                desc,
            });
        }
        Ok(blacklists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_endpoint_for() {
        let http_config = HttpConfig {
            url: "http://127.0.0.1:8080/auth".to_string(),
            method: "POST".to_string(),
            query_user: "users".to_string(),
            query_acl: "/acls".to_string(),
            query_blacklist: "http://127.0.0.1:8081/blacklists".to_string(),
            headers: None,
            body: None,
        };

        let adapter = HttpAuthStorageAdapter::new(http_config);
        assert_eq!(
            adapter.endpoint_for(&adapter.config.query_user),
            "http://127.0.0.1:8080/auth/users"
        );
        assert_eq!(
            adapter.endpoint_for(&adapter.config.query_acl),
            "http://127.0.0.1:8080/auth/acls"
        );
        assert_eq!(
            adapter.endpoint_for(&adapter.config.query_blacklist),
            "http://127.0.0.1:8081/blacklists"
        );
    }

    #[test]
    fn test_extract_items() {
        let payload = serde_json::json!({
            "users": [
                {"username": "u1", "password": "p1"},
                {"username": "u2", "password": "p2"}
            ]
        });
        let items =
            HttpAuthStorageAdapter::extract_items(payload, HttpAuthStorageAdapter::RESOURCE_USER)
                .unwrap();
        assert_eq!(items.len(), 2);
    }
}
