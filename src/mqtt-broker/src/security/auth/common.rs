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

use crate::core::constant::WILDCARD_RESOURCE;
use ipnet::IpNet;
use std::{net::IpAddr, str::FromStr};
use tracing::warn;

pub fn ip_match(source_ip_addr: &str, ip_role: &str) -> bool {
    // Check wildcard first
    if ip_role == WILDCARD_RESOURCE {
        return true;
    }

    // Parse source IP address
    let source_ip = match source_ip_addr.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(_) => {
            warn!("Invalid source IP address format: {}", source_ip_addr);
            return false;
        }
    };

    // Fast path: exact string match after validating source IP
    // This avoids parsing ip_role if it's exactly the same
    if source_ip_addr == ip_role {
        // Verify ip_role is also a valid IP to avoid false positives
        if ip_role.parse::<IpAddr>().is_ok() {
            return true;
        }
    }

    // Try CIDR match
    match IpNet::from_str(ip_role) {
        Ok(ip_cidr) => ip_cidr.contains(&source_ip),
        Err(_) => {
            // ip_role is neither a valid IP nor valid CIDR
            warn!(
                "Invalid IP pattern in blacklist: '{}' (not a valid IP or CIDR)",
                ip_role
            );
            false
        }
    }
}

pub fn topic_match(topic_name: &str, match_topic_name: &str) -> bool {
    if match_topic_name == WILDCARD_RESOURCE {
        return true;
    }
    topic_name == match_topic_name
}

#[cfg(test)]
mod test {
    use crate::{
        core::constant::WILDCARD_RESOURCE,
        security::auth::common::{ip_match, topic_match},
    };

    #[tokio::test]
    pub async fn topic_match_test() {
        let topic_name = "t1";
        let match_topic_name = WILDCARD_RESOURCE.to_string();
        assert!(topic_match(topic_name, &match_topic_name));
        assert!(topic_match(topic_name, topic_name));
        assert!(!topic_match(topic_name, "v1"));
    }

    #[tokio::test]
    pub async fn ip_match_test() {
        let source_ip = "127.0.0.1";
        let match_ip = WILDCARD_RESOURCE;
        assert!(ip_match(source_ip, match_ip));

        assert!(ip_match(source_ip, source_ip));
        assert!(!ip_match(source_ip, "192.1.1.1"));
        assert!(ip_match(source_ip, "127.0.0.1/24"));
    }

    #[tokio::test]
    pub async fn ip_match_invalid_source_test() {
        // Invalid source IP should return false
        assert!(!ip_match("not-an-ip", "127.0.0.1"));
        assert!(!ip_match("not-an-ip", "not-an-ip")); // Security: should not match invalid IPs
        assert!(!ip_match("", "127.0.0.1"));
    }

    #[tokio::test]
    pub async fn ip_match_invalid_pattern_test() {
        // Valid source IP but invalid pattern should return false
        assert!(!ip_match("127.0.0.1", "not-a-cidr"));
        assert!(!ip_match("127.0.0.1", "999.999.999.999"));
    }

    #[tokio::test]
    pub async fn ip_match_cidr_test() {
        // Test various CIDR ranges
        assert!(ip_match("192.168.1.100", "192.168.1.0/24"));
        assert!(ip_match("192.168.1.1", "192.168.0.0/16"));
        assert!(!ip_match("192.168.2.1", "192.168.1.0/24"));

        // Single IP as CIDR /32
        assert!(ip_match("10.0.0.1", "10.0.0.1/32"));
        assert!(!ip_match("10.0.0.2", "10.0.0.1/32"));
    }

    #[tokio::test]
    pub async fn ip_match_ipv6_test() {
        // Test IPv6 addresses
        assert!(ip_match("::1", "::1"));
        assert!(ip_match("2001:db8::1", "2001:db8::/32"));
        assert!(!ip_match("2001:db8::1", "2001:db9::/32"));
    }
}
