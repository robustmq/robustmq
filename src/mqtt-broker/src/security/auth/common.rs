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

pub fn ip_match(source_ip_addr: &str, ip_role: &str) -> bool {
    if ip_role == WILDCARD_RESOURCE {
        return true;
    }
    if source_ip_addr == ip_role {
        return true;
    }
    if let Ok(ip) = source_ip_addr.parse::<IpAddr>() {
        if let Ok(ip_cidr) = IpNet::from_str(ip_role) {
            return ip_cidr.contains(&ip);
        }
    }
    false
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
}
