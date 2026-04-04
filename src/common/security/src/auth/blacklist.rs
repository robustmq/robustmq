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

use crate::{auth::common::ip_match, manager::SecurityManager};
use common_base::error::common::CommonError;
use common_base::tools::now_second;
use regex::Regex;
use std::sync::Arc;
use tracing::{info, warn};

pub fn is_user_blacklisted(
    security_manager: &Arc<SecurityManager>,
    tenant: &str,
    user: &str,
) -> bool {
    let now = now_second();
    let meta = &security_manager.metadata;

    if let Some(tenant_map) = meta.blacklist_user.get(tenant) {
        if let Some(data) = tenant_map.get(user) {
            if is_active(data.end_time, now) {
                info!(username = %user, end_time = data.end_time, "Connection blocked by exact user blacklist");
                return true;
            }
        }
    }

    if let Some(list) = meta.blacklist_user_match.get(tenant) {
        for raw in list.iter() {
            if is_active(raw.end_time, now) && is_wildcard_pattern_match(user, &raw.resource_name) {
                info!(username = %user, pattern = %raw.resource_name, "Connection blocked by user wildcard blacklist");
                return true;
            }
        }
    }

    false
}

pub fn is_client_id_blacklisted(
    security_manager: &Arc<SecurityManager>,
    tenant: &str,
    client_id: &str,
) -> bool {
    let now = now_second();
    let meta = &security_manager.metadata;

    if let Some(tenant_map) = meta.blacklist_client_id.get(tenant) {
        if let Some(data) = tenant_map.get(client_id) {
            if is_active(data.end_time, now) {
                info!(client_id = %client_id, end_time = data.end_time, "Connection blocked by exact client_id blacklist");
                return true;
            }
        }
    }

    if let Some(list) = meta.blacklist_client_id_match.get(tenant) {
        for raw in list.iter() {
            if is_active(raw.end_time, now)
                && is_wildcard_pattern_match(client_id, &raw.resource_name)
            {
                info!(client_id = %client_id, pattern = %raw.resource_name, "Connection blocked by client_id wildcard blacklist");
                return true;
            }
        }
    }

    false
}

pub fn is_ip_blacklisted(
    security_manager: &Arc<SecurityManager>,
    tenant: &str,
    source_ip: &str,
) -> Result<bool, CommonError> {
    let now = now_second();
    let meta = &security_manager.metadata;

    if let Some(tenant_map) = meta.blacklist_ip.get(tenant) {
        if let Some(data) = tenant_map.get(source_ip) {
            if is_active(data.end_time, now) {
                info!(source_ip = %source_ip, end_time = data.end_time, "Connection blocked by exact IP blacklist");
                return Ok(true);
            }
        }
    }

    if let Some(list) = meta.blacklist_ip_match.get(tenant) {
        for raw in list.iter() {
            if is_active(raw.end_time, now) && ip_match(source_ip, &raw.resource_name)? {
                info!(source_ip = %source_ip, pattern = %raw.resource_name, "Connection blocked by IP pattern blacklist");
                return Ok(true);
            }
        }
    }

    Ok(false)
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
    end_time == 0 || end_time > now
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

#[cfg(test)]
mod tests {
    use super::{
        is_client_id_blacklisted, is_ip_blacklisted, is_user_blacklisted, is_wildcard_pattern_match,
    };
    use crate::manager::SecurityManager;
    use common_base::tools::now_second;
    use metadata_struct::auth::blacklist::{EnumBlackListType, SecurityBlackList};
    use std::sync::Arc;

    fn make_blacklist(
        tenant: &str,
        resource_name: &str,
        blacklist_type: EnumBlackListType,
        end_time: u64,
    ) -> SecurityBlackList {
        SecurityBlackList {
            name: resource_name.to_string(),
            tenant: tenant.to_string(),
            blacklist_type,
            resource_name: resource_name.to_string(),
            end_time,
            desc: String::new(),
        }
    }

    #[test]
    fn test_wildcard_pattern_match() {
        assert!(is_wildcard_pattern_match("admin", "admin*"));
        assert!(is_wildcard_pattern_match("admin_1", "admin*"));
        assert!(is_wildcard_pattern_match("admin", "adm?n"));
        assert!(!is_wildcard_pattern_match("root", "admin*"));
    }

    #[test]
    fn test_is_user_blacklisted() {
        let sm = Arc::new(SecurityManager::new());
        let tenant = "t1";
        let future = now_second() + 9999;
        let past = now_second() - 1;

        sm.metadata.add_blacklist(make_blacklist(
            tenant,
            "alice",
            EnumBlackListType::User,
            future,
        ));
        assert!(is_user_blacklisted(&sm, tenant, "alice"));

        sm.metadata
            .add_blacklist(make_blacklist(tenant, "bob", EnumBlackListType::User, past));
        assert!(!is_user_blacklisted(&sm, tenant, "bob"));

        sm.metadata.add_blacklist(make_blacklist(
            tenant,
            "permanent",
            EnumBlackListType::User,
            0,
        ));
        assert!(is_user_blacklisted(&sm, tenant, "permanent"));

        sm.metadata.add_blacklist(make_blacklist(
            tenant,
            "tmp_*",
            EnumBlackListType::UserMatch,
            future,
        ));
        assert!(is_user_blacklisted(&sm, tenant, "tmp_test"));
        assert!(!is_user_blacklisted(&sm, tenant, "normal_user"));
    }

    #[test]
    fn test_is_client_id_blacklisted() {
        let sm = Arc::new(SecurityManager::new());
        let tenant = "t1";
        let future = now_second() + 9999;
        let past = now_second() - 1;

        sm.metadata.add_blacklist(make_blacklist(
            tenant,
            "dev-001",
            EnumBlackListType::ClientId,
            future,
        ));
        assert!(is_client_id_blacklisted(&sm, tenant, "dev-001"));

        sm.metadata.add_blacklist(make_blacklist(
            tenant,
            "dev-002",
            EnumBlackListType::ClientId,
            past,
        ));
        assert!(!is_client_id_blacklisted(&sm, tenant, "dev-002"));

        sm.metadata.add_blacklist(make_blacklist(
            tenant,
            "tmp_*",
            EnumBlackListType::ClientIdMatch,
            future,
        ));
        assert!(is_client_id_blacklisted(&sm, tenant, "tmp_sensor"));
        assert!(!is_client_id_blacklisted(&sm, tenant, "prod_sensor"));
    }

    #[test]
    fn test_is_ip_blacklisted() {
        let sm = Arc::new(SecurityManager::new());
        let tenant = "t1";
        let future = now_second() + 9999;

        sm.metadata.add_blacklist(make_blacklist(
            tenant,
            "1.2.3.4",
            EnumBlackListType::Ip,
            future,
        ));
        assert!(is_ip_blacklisted(&sm, tenant, "1.2.3.4").unwrap());
        assert!(!is_ip_blacklisted(&sm, tenant, "1.2.3.5").unwrap());

        sm.metadata.add_blacklist(make_blacklist(
            tenant,
            "10.0.0.0/24",
            EnumBlackListType::IPCIDR,
            future,
        ));
        assert!(is_ip_blacklisted(&sm, tenant, "10.0.0.100").unwrap());
        assert!(!is_ip_blacklisted(&sm, tenant, "10.0.1.1").unwrap());
    }
}
