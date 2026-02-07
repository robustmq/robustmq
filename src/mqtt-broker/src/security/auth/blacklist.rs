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

use crate::{
    core::{cache::MQTTCacheManager, error::MqttBrokerError},
    security::auth::common::ip_match,
};
use common_base::tools::now_second;
use protocol::mqtt::common::Login;
use regex::Regex;
use std::sync::Arc;
use tracing::{info, warn};

/// Extract IP address from "IP:Port" format string
/// Handles both IPv4 (127.0.0.1:8080) and IPv6 ([::1]:8080) formats
/// Returns a String containing just the IP part
fn extract_ip_from_addr(addr: &str) -> String {
    use std::net::SocketAddr;

    // Try parsing as SocketAddr first (handles both IPv4 and IPv6)
    if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
        return socket_addr.ip().to_string();
    }

    // Fallback: try simple IPv4 "ip:port" split
    // Only split on the last colon to handle potential IPv6 without brackets
    if let Some((ip, _port)) = addr.rsplit_once(':') {
        // Verify it looks like an IPv4 address
        if ip.split('.').count() == 4 {
            return ip.to_string();
        }
    }

    // If all parsing fails, return the original address
    warn!("Failed to extract IP from address: {}", addr);
    addr.to_string()
}

/// Convert wildcard pattern to regex pattern
/// Converts shell/glob-style wildcards to regex:
/// - '*' becomes '.*' (match any characters)
/// - '?' becomes '.' (match single character)
/// - Other regex special chars are escaped
fn wildcard_to_regex(pattern: &str) -> String {
    let mut regex_pattern = String::with_capacity(pattern.len() * 2);

    for ch in pattern.chars() {
        match ch {
            '*' => regex_pattern.push_str(".*"),
            '?' => regex_pattern.push('.'),
            // Escape regex special characters
            '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                regex_pattern.push('\\');
                regex_pattern.push(ch);
            }
            _ => regex_pattern.push(ch),
        }
    }

    regex_pattern
}

pub fn is_blacklist(
    cache_manager: &Arc<MQTTCacheManager>,
    client_id: &str,
    source_ip_addr: &str,
    login: &Option<Login>,
) -> Result<bool, MqttBrokerError> {
    // todo: I believe this code can be refactored using the Chain of Responsibility pattern.
    let now = now_second();
    if let Some(log) = login {
        let login_user = log.username.to_string();
        // check user blacklist
        if !login_user.is_empty() {
            println!(
                "blacklist_user:{:?}",
                cache_manager.acl_metadata.blacklist_user.clone()
            );
            if let Some(data) = cache_manager.acl_metadata.blacklist_user.get(&login_user) {
                if data.end_time > now {
                    info!("user blacklist banned,user:{}", login_user);
                    return Ok(true);
                }
            }
        }

        if !login_user.is_empty() {
            if let Some(data) = cache_manager.acl_metadata.get_blacklist_user_match() {
                for raw in data {
                    // Check expiration first (cheap operation)
                    if raw.end_time <= now {
                        continue;
                    }

                    // Convert wildcard pattern to regex
                    let regex_pattern = format!("^{}$", wildcard_to_regex(&raw.resource_name));

                    // Then perform regex match (expensive operation)
                    match Regex::new(&regex_pattern) {
                        Ok(re) => {
                            if re.is_match(&login_user) {
                                info!(
                                    "user blacklist banned by match, user:{}, pattern:{}",
                                    login_user, raw.resource_name
                                );
                                return Ok(true);
                            }
                        }
                        Err(e) => {
                            // Log error but don't fail the entire check
                            warn!(
                                "Invalid wildcard pattern in user blacklist: {}, error: {}",
                                raw.resource_name, e
                            );
                            continue;
                        }
                    }
                }
            }
        }
    }

    // check client_id blacklist
    if let Some(data) = cache_manager
        .acl_metadata
        .blacklist_client_id
        .get(client_id)
    {
        if data.end_time > now {
            info!("client_id blacklist banned, client_id:{}", client_id);
            return Ok(true);
        }
    }

    if let Some(data) = cache_manager.acl_metadata.get_blacklist_client_id_match() {
        for raw in data {
            // Check expiration first (cheap operation)
            if raw.end_time <= now {
                continue;
            }

            // Convert wildcard pattern to regex
            let regex_pattern = format!("^{}$", wildcard_to_regex(&raw.resource_name));

            // Then perform regex match (expensive operation)
            match Regex::new(&regex_pattern) {
                Ok(re) => {
                    if re.is_match(client_id) {
                        info!(
                            "client_id blacklist banned by match, client_id:{}, pattern:{}",
                            client_id, raw.resource_name
                        );
                        return Ok(true);
                    }
                }
                Err(e) => {
                    // Log error but don't fail the entire check
                    warn!(
                        "Invalid wildcard pattern in client_id blacklist: {}, error: {}",
                        raw.resource_name, e
                    );
                    continue;
                }
            }
        }
    }

    // check ip blacklist
    // Extract IP from "IP:Port" format (e.g., "127.0.0.1:53836" -> "127.0.0.1")
    let source_ip = extract_ip_from_addr(source_ip_addr);

    // Check exact IP match
    if let Some(data) = cache_manager.acl_metadata.blacklist_ip.get(&source_ip) {
        if data.end_time > now {
            info!(
                "ip blacklist banned,source_ip_addr:{} (extracted IP: {})",
                source_ip_addr, &source_ip
            );
            return Ok(true);
        }
    }

    // Check IP pattern match (CIDR, etc.)
    if let Some(data) = cache_manager.acl_metadata.get_blacklist_ip_match() {
        for raw in data {
            // Check expiration first (cheap operation)
            if raw.end_time <= now {
                continue;
            }

            // Then perform IP match (relatively expensive operation)
            if ip_match(&source_ip, &raw.resource_name) {
                info!(
                    "ip blacklist banned by match,source_ip_addr:{} (extracted IP: {}), pattern:{}",
                    source_ip_addr, &source_ip, &raw.resource_name
                );
                return Ok(true);
            }
        }
    }

    Ok(false)
}

#[cfg(test)]
mod test {
    use super::{extract_ip_from_addr, is_blacklist, wildcard_to_regex};
    use crate::core::tool::test_build_mqtt_cache_manager;
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
    use common_base::tools::{local_hostname, now_second};
    use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
    use metadata_struct::mqtt::user::MqttUser;
    use protocol::mqtt::common::Login;
    use regex::Regex;

    #[tokio::test]
    async fn test_all_blacklist_types() {
        let cache_manager = test_build_mqtt_cache_manager().await;

        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "pass123".to_string(),
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };
        cache_manager.add_user(user.clone());

        let client_id = "client_id-1".to_string();
        let source_ip_addr = local_hostname();
        let login = Some(Login {
            username: user.username.clone(),
            password: user.password.clone(),
        });

        // Test no blacklist
        assert!(!is_blacklist(&cache_manager, &client_id, &source_ip_addr, &login).unwrap());

        let test_cases = vec![
            (MqttAclBlackListType::User, user.username.clone()),
            (MqttAclBlackListType::ClientId, client_id.clone()),
            (MqttAclBlackListType::Ip, source_ip_addr.clone()),
            (MqttAclBlackListType::UserMatch, "lobo*".to_string()),
            (MqttAclBlackListType::ClientIdMatch, "client*".to_string()),
            (MqttAclBlackListType::IPCIDR, "127.0.0.0/24".to_string()),
        ];

        for (blacklist_type, resource_name) in test_cases {
            let blacklist = MqttAclBlackList {
                blacklist_type,
                resource_name,
                end_time: now_second() + 100,
                desc: format!("Test {:?}", blacklist_type),
            };
            cache_manager.add_blacklist(blacklist.clone());

            assert!(
                is_blacklist(&cache_manager, &client_id, &source_ip_addr, &login).unwrap(),
                "Failed for {:?}",
                blacklist_type
            );

            cache_manager.remove_blacklist(blacklist);
        }
    }

    #[tokio::test]
    async fn test_blacklist_with_ip_port() {
        let cache_manager = test_build_mqtt_cache_manager().await;

        let user = MqttUser {
            username: "testuser".to_string(),
            password: "pass123".to_string(),
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };
        cache_manager.add_user(user.clone());

        let test_cases = vec![
            ("127.0.0.1:53836", MqttAclBlackListType::Ip, "127.0.0.1"),
            (
                "192.168.1.100:8080",
                MqttAclBlackListType::IPCIDR,
                "192.168.1.0/24",
            ),
        ];

        for (source_ip_addr, blacklist_type, resource_name) in test_cases {
            let blacklist = MqttAclBlackList {
                blacklist_type,
                resource_name: resource_name.to_string(),
                end_time: now_second() + 100,
                desc: "Test IP with port".to_string(),
            };
            cache_manager.add_blacklist(blacklist.clone());

            let login = Some(Login {
                username: user.username.clone(),
                password: user.password.clone(),
            });

            assert!(
                is_blacklist(&cache_manager, "test-client", source_ip_addr, &login).unwrap(),
                "Failed for {:?} with IP:Port {}",
                blacklist_type,
                source_ip_addr
            );

            cache_manager.remove_blacklist(blacklist);
        }
    }

    #[test]
    fn test_extract_ip_from_addr() {
        assert_eq!(extract_ip_from_addr("127.0.0.1:53836"), "127.0.0.1");
        assert_eq!(extract_ip_from_addr("192.168.1.100:8080"), "192.168.1.100");
        assert_eq!(extract_ip_from_addr("127.0.0.1"), "127.0.0.1");
        assert_eq!(extract_ip_from_addr("[::1]:8080"), "::1");
        assert_eq!(extract_ip_from_addr("[2001:db8::1]:1883"), "2001:db8::1");
        assert_eq!(extract_ip_from_addr("localhost:8080"), "localhost:8080");
    }

    #[test]
    fn test_wildcard_to_regex() {
        assert_eq!(wildcard_to_regex("test*"), "test.*");
        assert_eq!(wildcard_to_regex("test?"), "test.");
        assert_eq!(wildcard_to_regex("test.example"), "test\\.example");
        assert_eq!(wildcard_to_regex("test_*.example"), "test_.*\\.example");

        let pattern = "test_client_*";
        let regex_str = format!("^{}$", wildcard_to_regex(pattern));
        let re = Regex::new(&regex_str).unwrap();
        assert!(re.is_match("test_client_001"));
        assert!(re.is_match("test_client_anything"));
        assert!(!re.is_match("test_client"));

        let pattern2 = "test_?_end";
        let regex_str2 = format!("^{}$", wildcard_to_regex(pattern2));
        let re2 = Regex::new(&regex_str2).unwrap();
        assert!(re2.is_match("test_a_end"));
        assert!(!re2.is_match("test_ab_end"));
    }
}
