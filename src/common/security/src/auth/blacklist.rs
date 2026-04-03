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
    core::{cache::MQTTCacheManager, error::CommonError},
    security::auth::common::ip_match,
};
use common_base::tools::now_second;
use protocol::mqtt::common::Login;
use regex::Regex;
use std::sync::Arc;
use tracing::{info, warn};

pub fn is_connection_blacklisted(
    cache_manager: &Arc<MQTTCacheManager>,
    client_id: &str,
    source_ip_addr: &str,
    login: &Option<Login>,
) -> Result<bool, CommonError> {
    let now = now_second();

    Ok(is_user_blacklisted(cache_manager, login, now)
        || is_client_id_blacklisted(cache_manager, client_id, now)
        || is_ip_blacklisted(cache_manager, source_ip_addr, now))
}

fn extract_ip_from_addr(addr: &str) -> String {
    use std::net::SocketAddr;

    if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
        return socket_addr.ip().to_string();
    }

    if let Some((ip, _port)) = addr.rsplit_once(':') {
        if ip.split('.').count() == 4 {
            return ip.to_string();
        }
    }

    warn!(source_addr = %addr, "Failed to extract IP from source address");
    addr.to_string()
}

fn wildcard_to_regex(pattern: &str) -> String {
    let mut regex_pattern = String::with_capacity(pattern.len() * 2);

    for ch in pattern.chars() {
        match ch {
            '*' => regex_pattern.push_str(".*"),
            '?' => regex_pattern.push('.'),
            '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                regex_pattern.push('\\');
                regex_pattern.push(ch);
            }
            _ => regex_pattern.push(ch),
        }
    }

    regex_pattern
}

fn is_active(end_time: u64, now: u64) -> bool {
    end_time > now
}

fn is_wildcard_pattern_match(target: &str, pattern: &str) -> bool {
    let regex_pattern = format!("^{}$", wildcard_to_regex(pattern));
    match Regex::new(&regex_pattern) {
        Ok(re) => re.is_match(target),
        Err(e) => {
            warn!(pattern = %pattern, error = %e, "Invalid wildcard pattern");
            false
        }
    }
}

fn is_user_blacklisted(
    cache_manager: &Arc<MQTTCacheManager>,
    login: &Option<Login>,
    now: u64,
) -> bool {
    let Some(login) = login else {
        return false;
    };
    let login_user = login.username.to_string();
    if login_user.is_empty() {
        return false;
    }

    // Check exact match across all tenants
    for tenant_entry in cache_manager.acl_metadata.blacklist_user.iter() {
        if let Some(data) = tenant_entry.value().get(&login_user) {
            if is_active(data.end_time, now) {
                info!(
                    username = %login_user,
                    end_time = data.end_time,
                    now = now,
                    "Connection blocked by exact user blacklist"
                );
                return true;
            }
        }
    }

    // Check wildcard match across all tenants
    for raw in cache_manager.acl_metadata.get_blacklist_user_match() {
        if !is_active(raw.end_time, now) {
            continue;
        }
        if is_wildcard_pattern_match(&login_user, &raw.resource_name) {
            info!(
                username = %login_user,
                pattern = %raw.resource_name,
                end_time = raw.end_time,
                now = now,
                "Connection blocked by user wildcard blacklist"
            );
            return true;
        }
    }

    false
}

fn is_client_id_blacklisted(
    cache_manager: &Arc<MQTTCacheManager>,
    client_id: &str,
    now: u64,
) -> bool {
    // Check exact match across all tenants
    for tenant_entry in cache_manager.acl_metadata.blacklist_client_id.iter() {
        if let Some(data) = tenant_entry.value().get(client_id) {
            if is_active(data.end_time, now) {
                info!(
                    client_id = %client_id,
                    end_time = data.end_time,
                    now = now,
                    "Connection blocked by exact client_id blacklist"
                );
                return true;
            }
        }
    }

    // Check wildcard match across all tenants
    for raw in cache_manager.acl_metadata.get_blacklist_client_id_match() {
        if !is_active(raw.end_time, now) {
            continue;
        }
        if is_wildcard_pattern_match(client_id, &raw.resource_name) {
            info!(
                client_id = %client_id,
                pattern = %raw.resource_name,
                end_time = raw.end_time,
                now = now,
                "Connection blocked by client_id wildcard blacklist"
            );
            return true;
        }
    }

    false
}

fn is_ip_blacklisted(
    cache_manager: &Arc<MQTTCacheManager>,
    source_ip_addr: &str,
    now: u64,
) -> bool {
    let source_ip = extract_ip_from_addr(source_ip_addr);

    // Check exact match across all tenants
    for tenant_entry in cache_manager.acl_metadata.blacklist_ip.iter() {
        if let Some(data) = tenant_entry.value().get(&source_ip) {
            if is_active(data.end_time, now) {
                info!(
                    source_ip_addr = %source_ip_addr,
                    source_ip = %source_ip,
                    end_time = data.end_time,
                    now = now,
                    "Connection blocked by exact IP blacklist"
                );
                return true;
            }
        }
    }

    // Check CIDR/wildcard match across all tenants
    for raw in cache_manager.acl_metadata.get_blacklist_ip_match() {
        if !is_active(raw.end_time, now) {
            continue;
        }
        if ip_match(&source_ip, &raw.resource_name) {
            info!(
                source_ip_addr = %source_ip_addr,
                source_ip = %source_ip,
                pattern = %raw.resource_name,
                end_time = raw.end_time,
                now = now,
                "Connection blocked by IP pattern blacklist"
            );
            return true;
        }
    }

    false
}

#[cfg(test)]
mod test {
    use super::{extract_ip_from_addr, is_connection_blacklisted, wildcard_to_regex};
    use crate::core::tool::test_build_mqtt_cache_manager;
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::EnumBlackListType;
    use common_base::tools::{local_hostname, now_second};
    use metadata_struct::auth::blacklist::SecurityBlackList;
    use metadata_struct::auth::user::SecurityUser;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use protocol::mqtt::common::Login;
    use regex::Regex;

    fn build_login(username: &str, password: &str) -> Option<Login> {
        Some(Login {
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    mod extract_ip_from_addr_cases {
        use super::extract_ip_from_addr;

        #[test]
        fn parses_ipv4_and_ipv6_and_fallback() {
            let cases = [
                ("127.0.0.1:53836", "127.0.0.1"),
                ("192.168.1.100:8080", "192.168.1.100"),
                ("127.0.0.1", "127.0.0.1"),
                ("[::1]:8080", "::1"),
                ("[2001:db8::1]:1883", "2001:db8::1"),
                ("localhost:8080", "localhost:8080"),
            ];

            for (input, expected) in cases {
                assert_eq!(extract_ip_from_addr(input), expected);
            }
        }
    }

    mod wildcard_to_regex_cases {
        use super::{wildcard_to_regex, Regex};

        #[test]
        fn converts_wildcards_and_matches() {
            assert_eq!(wildcard_to_regex("test*"), "test.*");
            assert_eq!(wildcard_to_regex("test?"), "test.");
            assert_eq!(wildcard_to_regex("test.example"), "test\\.example");

            let re = Regex::new(&format!("^{}$", wildcard_to_regex("test_client_*"))).unwrap();
            assert!(re.is_match("test_client_001"));
            assert!(!re.is_match("test_client"));

            let re2 = Regex::new(&format!("^{}$", wildcard_to_regex("test_?_end"))).unwrap();
            assert!(re2.is_match("test_a_end"));
            assert!(!re2.is_match("test_ab_end"));
        }
    }

    mod is_connection_blacklisted_cases {
        use super::{
            build_login, is_connection_blacklisted, local_hostname, now_second,
            test_build_mqtt_cache_manager, SecurityBlackList, EnumBlackListType, SecurityUser,
            DEFAULT_TENANT,
        };

        #[tokio::test]
        async fn blocks_on_all_blacklist_types() {
            let cache_manager = test_build_mqtt_cache_manager().await;
            let user = SecurityUser {
                tenant: DEFAULT_TENANT.to_string(),
                username: "loboxu".to_string(),
                password: "pass123".to_string(),
                salt: None,
                is_superuser: false,
                create_time: now_second(),
            };
            cache_manager.add_user(user.clone());

            let client_id = "client_id-1";
            let source_ip_addr = local_hostname();
            let login = build_login(&user.username, &user.password);

            assert!(
                !is_connection_blacklisted(&cache_manager, client_id, &source_ip_addr, &login)
                    .unwrap()
            );

            let cases = vec![
                (EnumBlackListType::User, user.username.clone()),
                (EnumBlackListType::ClientId, client_id.to_string()),
                (EnumBlackListType::Ip, source_ip_addr.clone()),
                (EnumBlackListType::UserMatch, "lobo*".to_string()),
                (EnumBlackListType::ClientIdMatch, "client*".to_string()),
                (EnumBlackListType::IPCIDR, "127.0.0.0/24".to_string()),
            ];

            for (blacklist_type, resource_name) in cases {
                let blacklist = SecurityBlackList {
                    name: format!("test-blacklist-{resource_name}"),
                    tenant: DEFAULT_TENANT.to_string(),
                    blacklist_type,
                    resource_name,
                    end_time: now_second() + 100,
                    desc: "test".to_string(),
                };
                cache_manager.add_blacklist(blacklist.clone());
                assert!(is_connection_blacklisted(
                    &cache_manager,
                    client_id,
                    &source_ip_addr,
                    &login
                )
                .unwrap());
                cache_manager.remove_blacklist(blacklist);
            }
        }

        #[tokio::test]
        async fn supports_ip_port_input() {
            let cache_manager = test_build_mqtt_cache_manager().await;
            let user = SecurityUser {
                tenant: DEFAULT_TENANT.to_string(),
                username: "testuser".to_string(),
                password: "pass123".to_string(),
                salt: None,
                is_superuser: false,
                create_time: now_second(),
            };
            cache_manager.add_user(user.clone());
            let login = build_login(&user.username, &user.password);

            let cases = vec![
                ("127.0.0.1:53836", EnumBlackListType::Ip, "127.0.0.1"),
                (
                    "192.168.1.100:8080",
                    EnumBlackListType::IPCIDR,
                    "192.168.1.0/24",
                ),
            ];

            for (source_ip_addr, blacklist_type, resource_name) in cases {
                let blacklist = SecurityBlackList {
                    name: format!("test-blacklist-{resource_name}"),
                    tenant: DEFAULT_TENANT.to_string(),
                    blacklist_type,
                    resource_name: resource_name.to_string(),
                    end_time: now_second() + 100,
                    desc: "test".to_string(),
                };
                cache_manager.add_blacklist(blacklist.clone());
                assert!(is_connection_blacklisted(
                    &cache_manager,
                    "test-client",
                    source_ip_addr,
                    &login
                )
                .unwrap());
                cache_manager.remove_blacklist(blacklist);
            }
        }
    }
}
