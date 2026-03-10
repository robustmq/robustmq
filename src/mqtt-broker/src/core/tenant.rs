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

use crate::core::{cache::MQTTCacheManager, error::MqttBrokerError};
use protocol::mqtt::common::{ConnectProperties, Login};
use std::sync::Arc;

pub const DEFAULT_TENANT: &str = "default";
pub const TENANT_SEPARATOR: &str = "@";
pub const TENANT_USER_PROPERTY_KEY: &str = "tenant";

pub fn get_tenant_info(
    cache_manager: &Arc<MQTTCacheManager>,
    client_id: &str,
    connect_properties: &Option<ConnectProperties>,
    login: &Option<Login>,
) -> Result<metadata_struct::tenant::Tenant, MqttBrokerError> {
    let tenant_name = decode_tenant_name(client_id, connect_properties, login)?;
    cache_manager
        .broker_cache
        .get_tenant(&tenant_name)
        .ok_or_else(|| MqttBrokerError::TenantNotFound(tenant_name))
}

fn decode_tenant_name(
    client_id: &str,
    connect_properties: &Option<ConnectProperties>,
    login: &Option<Login>,
) -> Result<String, MqttBrokerError> {
    if let Some(props) = connect_properties {
        for (k, v) in &props.user_properties {
            if k == TENANT_USER_PROPERTY_KEY && !v.is_empty() {
                return Ok(v.clone());
            }
        }
    }

    if let Some(login) = login {
        if let Some(tenant) = extract_prefix(&login.username) {
            return Ok(tenant);
        }
    }

    if let Some(tenant) = extract_prefix(client_id) {
        return Ok(tenant);
    }

    Ok(DEFAULT_TENANT.to_string())
}

fn extract_prefix(s: &str) -> Option<String> {
    if let Some((prefix, _)) = s.split_once(TENANT_SEPARATOR) {
        if !prefix.is_empty() {
            return Some(prefix.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::mqtt::common::ConnectProperties;

    #[test]
    fn test_priority_order() {
        let mut props = ConnectProperties::default();
        props.user_properties.push((
            TENANT_USER_PROPERTY_KEY.to_string(),
            "prop-tenant".to_string(),
        ));

        let result = decode_tenant_name(
            "cid-tenant@device",
            &Some(props),
            &Some(Login {
                username: "username-tenant@admin".to_string(),
                password: "pass".to_string(),
            }),
        )
        .unwrap();
        assert_eq!(result, "prop-tenant");
    }

    #[test]
    fn test_fallback_chain() {
        assert_eq!(
            decode_tenant_name("plain-device", &None, &None).unwrap(),
            DEFAULT_TENANT
        );
        assert_eq!(
            decode_tenant_name(
                "plain-device",
                &None,
                &Some(Login {
                    username: "biz@admin".to_string(),
                    password: "pass".to_string(),
                })
            )
            .unwrap(),
            "biz"
        );
        assert_eq!(
            decode_tenant_name("biz@device-001", &None, &None).unwrap(),
            "biz"
        );
    }
}
