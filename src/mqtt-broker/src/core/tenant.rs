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
use metadata_struct::tenant::DEFAULT_TENANT;
use protocol::mqtt::common::{ConnectProperties, Login};
use std::sync::Arc;

/// The separator used to embed tenant name inside client_id and username.
///
/// Convention: `<tenant>@<real_value>`
/// - client_id:  `"acme@device-001"` → tenant = `"acme"`, real client_id = `"device-001"`
/// - username:   `"acme@alice"`       → tenant = `"acme"`, real username   = `"alice"`
pub const TENANT_SEPARATOR: &str = "@";

/// The key used to carry tenant name in MQTT v5 CONNECT user-properties.
///
/// Clients can set a user-property `("tenant", "<tenant_name>")` in their
/// CONNECT packet instead of embedding the tenant in client_id / username.
pub const TENANT_USER_PROPERTY_KEY: &str = "tenant";

/// Resolve the tenant for an incoming CONNECT, then look it up in the cache.
///
/// Returns `TenantNotFound` if the resolved name does not match any known tenant.
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

/// Determine the tenant name from an MQTT CONNECT packet.
///
/// Three sources are tried in priority order (highest → lowest):
///
/// 1. **MQTT v5 user-property** (`"tenant"` key in CONNECT properties)
///    - Explicit, takes precedence over everything else.
///    - Example: user-property `("tenant", "acme")` → tenant = `"acme"`.
///
/// 2. **Username prefix** (the part before `@` in the username field)
///    - Format: `<tenant>@<real_username>`, e.g. `"acme@alice"` → tenant = `"acme"`.
///    - Skipped if login is absent or username contains no `@`.
///
/// 3. **Client-ID prefix** (the part before `@` in client_id)
///    - Format: `<tenant>@<real_client_id>`, e.g. `"acme@device-001"` → tenant = `"acme"`.
///    - Skipped if client_id contains no `@`.
///
/// 4. **Default tenant fallback**
///    - Used when none of the above sources yield a tenant.
fn decode_tenant_name(
    client_id: &str,
    connect_properties: &Option<ConnectProperties>,
    login: &Option<Login>,
) -> Result<String, MqttBrokerError> {
    // Priority 1: MQTT v5 user-property "tenant"
    if let Some(props) = connect_properties {
        for (k, v) in &props.user_properties {
            if k == TENANT_USER_PROPERTY_KEY && !v.is_empty() {
                return Ok(v.clone());
            }
        }
    }

    // Priority 2: username prefix — "acme@alice" → "acme"
    if let Some(login) = login {
        if let Some(tenant) = extract_prefix(&login.username) {
            return Ok(tenant);
        }
    }

    // Priority 3: client_id prefix — "acme@device-001" → "acme"
    if let Some(tenant) = extract_prefix(client_id) {
        return Ok(tenant);
    }

    // Priority 4: fall back to the default tenant
    Ok(DEFAULT_TENANT.to_string())
}

/// Extract the tenant from `<tenant>@<rest>`.
///
/// Returns `None` if the string contains no `@`, or if the prefix is empty
/// (e.g. `"@device"` has an empty tenant and is treated as "no tenant specified").
fn extract_prefix(s: &str) -> Option<String> {
    if let Some((prefix, _)) = s.split_once(TENANT_SEPARATOR) {
        if !prefix.is_empty() {
            return Some(prefix.to_string());
        }
    }
    None
}

/// Extract the real value from `<tenant>@<rest>`, stripping the tenant prefix.
///
/// If there is no `@`, the original string is returned as-is.
/// Examples:
/// - `"acme@alice"`      → `"alice"`
/// - `"alice"`           → `"alice"`
/// - `"@alice"`          → `"alice"`
fn extract_suffix(s: &str) -> String {
    if let Some((_, suffix)) = s.split_once(TENANT_SEPARATOR) {
        suffix.to_string()
    } else {
        s.to_string()
    }
}

/// Strip the tenant prefix from a username, returning the real username.
///
/// - `"acme@alice"` → `"alice"`
/// - `"alice"`      → `"alice"`
pub fn try_decode_username(username: &str) -> String {
    extract_suffix(username)
}

/// Strip the tenant prefix from a client_id, returning the real client_id.
///
/// - `"acme@device-001"` → `"device-001"`
/// - `"device-001"`      → `"device-001"`
pub fn try_decode_client_id(client_id: &str) -> String {
    extract_suffix(client_id)
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
    fn test_try_decode_username() {
        assert_eq!(try_decode_username("biz@admin"), "admin");
        assert_eq!(try_decode_username("admin"), "admin");
        assert_eq!(try_decode_username("@admin"), "admin");
    }

    #[test]
    fn test_try_decode_client_id() {
        assert_eq!(try_decode_client_id("biz@device-001"), "device-001");
        assert_eq!(try_decode_client_id("device-001"), "device-001");
        assert_eq!(try_decode_client_id("@device-001"), "device-001");
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
