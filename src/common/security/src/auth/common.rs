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

use crate::WILDCARD_RESOURCE;
use ipnet::IpNet;
use std::{net::IpAddr, str::FromStr};
use tracing::warn;

pub fn ip_match(source_ip_addr: &str, ip_role: &str) -> bool {
    if ip_role.is_empty() || ip_role == WILDCARD_RESOURCE {
        return true;
    }

    let source_ip = match source_ip_addr.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(_) => {
            warn!("Invalid source IP address format: {}", source_ip_addr);
            return false;
        }
    };

    if source_ip_addr == ip_role {
        if ip_role.parse::<IpAddr>().is_ok() {
            return true;
        }
    }

    match IpNet::from_str(ip_role) {
        Ok(ip_cidr) => ip_cidr.contains(&source_ip),
        Err(_) => {
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
        auth::common::{ip_match, topic_match},
        WILDCARD_RESOURCE,
    };

    #[test]
    fn topic_match_test() {
        assert!(topic_match("t1", WILDCARD_RESOURCE));
        assert!(topic_match("t1", "t1"));
        assert!(!topic_match("t1", "t2"));
    }

    #[test]
    fn ip_match_test() {
        let cases = [
            ("127.0.0.1", WILDCARD_RESOURCE, true),
            ("127.0.0.1", "127.0.0.1", true),
            ("127.0.0.1", "192.1.1.1", false),
            ("127.0.0.1", "127.0.0.1/24", true),
            ("10.0.0.2", "10.0.0.1/32", false),
            ("not-an-ip", "127.0.0.1", false),
            ("127.0.0.1", "not-a-cidr", false),
            ("127.0.0.1", "", true),
            ("2001:db8::1", "2001:db8::/32", true),
            ("2001:db8::1", "2001:db9::/32", false),
        ];

        for (source, pattern, expected) in cases {
            assert_eq!(ip_match(source, pattern), expected);
        }
    }
}
