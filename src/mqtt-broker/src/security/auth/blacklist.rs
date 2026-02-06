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
use metadata_struct::mqtt::connection::MQTTConnection;
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
    connection: &MQTTConnection,
) -> Result<bool, MqttBrokerError> {
    // todo: I believe this code can be refactored using the Chain of Responsibility pattern.
    let now = now_second();
    let user = connection.login_user.as_deref().unwrap_or("");
    println!("connection:{:?}", connection);
    // check user blacklist
    if !user.is_empty() {
        if let Some(data) = cache_manager.acl_metadata.blacklist_user.get(user) {
            if data.end_time > now {
                info!("user blacklist banned,user:{}", user);
                return Ok(true);
            }
        }
    }

    if !user.is_empty() {
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
                        if re.is_match(user) {
                            info!(
                                "user blacklist banned by match, user:{}, pattern:{}",
                                user, raw.resource_name
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

    // check client_id blacklist
    if let Some(data) = cache_manager
        .acl_metadata
        .blacklist_client_id
        .get(&connection.client_id)
    {
        if data.end_time > now {
            info!(
                "client_id blacklist banned, client_id:{}",
                &connection.client_id
            );
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
                    if re.is_match(&connection.client_id) {
                        info!(
                            "client_id blacklist banned by match, client_id:{}, pattern:{}",
                            &connection.client_id, raw.resource_name
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
    let source_ip = extract_ip_from_addr(&connection.source_ip_addr);
    println!("source_ip:{}", source_ip);

    // Check exact IP match
    if let Some(data) = cache_manager.acl_metadata.blacklist_ip.get(&source_ip) {
        if data.end_time > now {
            info!(
                "ip blacklist banned,source_ip_addr:{} (extracted IP: {})",
                &connection.source_ip_addr, &source_ip
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
                    &connection.source_ip_addr, &source_ip, &raw.resource_name
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
    use crate::core::cache::MQTTCacheManager;
    use crate::core::tool::test_build_mqtt_cache_manager;
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
    use common_base::tools::{local_hostname, now_second};
    use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
    use metadata_struct::mqtt::connection::{ConnectionConfig, MQTTConnection};
    use metadata_struct::mqtt::user::MqttUser;
    use regex::Regex;
    use std::sync::Arc;

    struct TestFixture {
        cache_manager: Arc<MQTTCacheManager>,
        connection: MQTTConnection,
        user: MqttUser,
    }

    async fn setup() -> TestFixture {
        let cache_manager = test_build_mqtt_cache_manager().await;

        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            salt: None,
            is_superuser: true,
            create_time: now_second(),
        };

        cache_manager.add_user(user.clone());

        let config = ConnectionConfig {
            connect_id: 1,
            client_id: "client_id-1".to_string(),
            receive_maximum: 3,
            max_packet_size: 3,
            topic_alias_max: 3,
            request_problem_info: 1,
            keep_alive: 2,
            source_ip_addr: local_hostname(),
            clean_session: false,
        };
        let mut connection = MQTTConnection::new(config);
        connection.login_success(user.username.clone());

        TestFixture {
            cache_manager,
            connection,
            user,
        }
    }

    fn assert_is_blacklisted(
        fixture: &TestFixture,
        blacklist_type: MqttAclBlackListType,
        resource_name: String,
    ) {
        let blacklist = MqttAclBlackList {
            blacklist_type,
            resource_name,
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        fixture.cache_manager.add_blacklist(blacklist);

        assert!(
            is_blacklist(&fixture.cache_manager, &fixture.connection).unwrap(),
            "Expected to be blacklisted but was not"
        );
    }

    #[tokio::test]
    async fn test_not_blacklist_default() {
        let fixture = setup().await;
        assert!(!is_blacklist(&fixture.cache_manager, &fixture.connection).unwrap_or(true))
    }

    #[tokio::test]
    async fn test_blacklist_client_id() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::ClientId,
            fixture.connection.client_id.clone(),
        );
    }

    #[tokio::test]
    async fn test_blacklist_user() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::User,
            fixture.user.username.clone(),
        );
    }

    #[tokio::test]
    async fn test_blacklist_ip() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::Ip,
            fixture.connection.source_ip_addr.clone(),
        );
    }

    #[tokio::test]
    async fn test_blacklist_client_id_match() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::ClientIdMatch,
            fixture.connection.client_id.clone(),
        );
    }

    #[tokio::test]
    async fn test_blacklist_user_match() {
        let fixture = setup().await;
        // Use wildcard pattern: "lobo*" matches "lobo" followed by any characters
        // (Previously used "lobo.*" which was regex syntax, now using glob/wildcard syntax)
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::UserMatch,
            "lobo*".to_string(), // Matches "loboxu"
        );
    }

    #[tokio::test]
    async fn test_blacklist_ip_cidr() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::IPCIDR,
            "127.0.0.0/24".to_string(),
        );
    }

    #[test]
    fn test_extract_ip_from_addr() {
        // Test IPv4 with port
        assert_eq!(extract_ip_from_addr("127.0.0.1:53836"), "127.0.0.1");
        assert_eq!(extract_ip_from_addr("192.168.1.100:8080"), "192.168.1.100");

        // Test IPv4 without port
        assert_eq!(extract_ip_from_addr("127.0.0.1"), "127.0.0.1");

        // Test IPv6 with port (bracketed format)
        assert_eq!(extract_ip_from_addr("[::1]:8080"), "::1");
        assert_eq!(extract_ip_from_addr("[2001:db8::1]:1883"), "2001:db8::1");

        // Test edge cases - non-IP hostname with port
        // Should return original since it can't parse as SocketAddr
        assert_eq!(extract_ip_from_addr("localhost:8080"), "localhost:8080");
    }

    #[test]
    fn test_wildcard_to_regex() {
        // Test wildcard '*' converts to '.*'
        assert_eq!(wildcard_to_regex("test*"), "test.*");
        assert_eq!(wildcard_to_regex("*test"), ".*test");
        assert_eq!(wildcard_to_regex("test_*_suffix"), "test_.*_suffix");

        // Test wildcard '?' converts to '.'
        assert_eq!(wildcard_to_regex("test?"), "test.");
        assert_eq!(wildcard_to_regex("test_?_suffix"), "test_._suffix");

        // Test regex special chars are escaped
        assert_eq!(wildcard_to_regex("test.example"), "test\\.example");
        assert_eq!(wildcard_to_regex("test+pattern"), "test\\+pattern");
        assert_eq!(wildcard_to_regex("test(123)"), "test\\(123\\)");
        assert_eq!(wildcard_to_regex("test[abc]"), "test\\[abc\\]");

        // Test combined patterns
        assert_eq!(wildcard_to_regex("test_*.example"), "test_.*\\.example");
        assert_eq!(wildcard_to_regex("user-?.test*"), "user-.\\.test.*");
    }

    #[test]
    fn test_wildcard_pattern_matching() {
        // Test case from user's issue
        let pattern = "test_client_d62392alko8dt5ceahag*";
        let regex_str = format!("^{}$", wildcard_to_regex(pattern));
        let re = Regex::new(&regex_str).unwrap();

        // Should match
        assert!(re.is_match("test_client_d62392alko8dt5ceahag_001"));
        assert!(re.is_match("test_client_d62392alko8dt5ceahag"));
        assert!(re.is_match("test_client_d62392alko8dt5ceahag_anything"));

        // Should not match
        assert!(!re.is_match("test_client_different"));
        assert!(!re.is_match("test_client_d62392alko8dt5ceaha")); // Missing 'g'
    }

    #[test]
    fn test_wildcard_question_mark() {
        let pattern = "test_?_end";
        let regex_str = format!("^{}$", wildcard_to_regex(pattern));
        let re = Regex::new(&regex_str).unwrap();

        // Should match (? matches exactly one char)
        assert!(re.is_match("test_a_end"));
        assert!(re.is_match("test_1_end"));
        assert!(re.is_match("test_X_end"));

        // Should not match
        assert!(!re.is_match("test__end")); // Empty between underscores
        assert!(!re.is_match("test_ab_end")); // Two chars instead of one
    }

    #[tokio::test]
    async fn test_blacklist_ip_with_port() {
        let cache_manager = test_build_mqtt_cache_manager().await;

        let user = MqttUser {
            username: "testuser".to_string(),
            password: "pass123".to_string(),
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };

        cache_manager.add_user(user.clone());

        // Create connection with IP:Port format (real-world scenario)
        let config = ConnectionConfig {
            connect_id: 1,
            client_id: "test-client".to_string(),
            receive_maximum: 3,
            max_packet_size: 3,
            topic_alias_max: 3,
            request_problem_info: 1,
            keep_alive: 2,
            source_ip_addr: "127.0.0.1:53836".to_string(), // IP with port
            clean_session: false,
        };
        let mut connection = MQTTConnection::new(config);
        connection.login_success(user.username.clone());

        // Blacklist the IP only (without port)
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::Ip,
            resource_name: "127.0.0.1".to_string(), // Just the IP
            end_time: now_second() + 100,
            desc: "Test IP blacklist with port".to_string(),
        };
        cache_manager.add_blacklist(blacklist);

        // Should match even though connection has port
        assert!(
            is_blacklist(&cache_manager, &connection).unwrap(),
            "IP blacklist should match when connection has port"
        );
    }

    #[tokio::test]
    async fn test_blacklist_ip_cidr_with_port() {
        let cache_manager = test_build_mqtt_cache_manager().await;

        let user = MqttUser {
            username: "testuser".to_string(),
            password: "pass123".to_string(),
            salt: None,
            is_superuser: false,
            create_time: now_second(),
        };

        cache_manager.add_user(user.clone());

        // Create connection with IP:Port format
        let config = ConnectionConfig {
            connect_id: 1,
            client_id: "test-client".to_string(),
            receive_maximum: 3,
            max_packet_size: 3,
            topic_alias_max: 3,
            request_problem_info: 1,
            keep_alive: 2,
            source_ip_addr: "192.168.1.100:8080".to_string(),
            clean_session: false,
        };
        let mut connection = MQTTConnection::new(config);
        connection.login_success(user.username.clone());

        // Blacklist via CIDR
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::IPCIDR,
            resource_name: "192.168.1.0/24".to_string(),
            end_time: now_second() + 100,
            desc: "Test CIDR blacklist with port".to_string(),
        };
        cache_manager.add_blacklist(blacklist);

        // Should match via CIDR even though connection has port
        assert!(
            is_blacklist(&cache_manager, &connection).unwrap(),
            "CIDR blacklist should match when connection has port"
        );
    }
}
