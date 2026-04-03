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
    auth::common::{ip_match, topic_match},
    manager::SecurityManager,
};
use metadata_struct::auth::acl::{EnumAclAction, EnumAclPermission, SecurityAcl};
use std::{net::SocketAddr, sync::Arc};

pub fn normalize_source_ip(source_ip_addr: &str) -> String {
    if let Ok(socket_addr) = source_ip_addr.parse::<SocketAddr>() {
        return socket_addr.ip().to_string();
    }
    if let Some((ip, _)) = source_ip_addr.rsplit_once(':') {
        if ip.split('.').count() == 4 {
            return ip.to_string();
        }
    }
    source_ip_addr.to_string()
}

pub fn is_user_acl_deny(
    security_manager: &Arc<SecurityManager>,
    topic_name: &str,
    tenant: &str,
    user: &str,
    source_ip: &str,
    action: &EnumAclAction,
) -> bool {
    if let Some(tenant_map) = security_manager.security_metadata.acl_user.get(tenant) {
        if let Some(acl_list) = tenant_map.get(user) {
            return check_for_deny(&acl_list, action, topic_name, source_ip);
        }
    }
    false
}

pub fn is_client_id_acl_deny(
    security_manager: &Arc<SecurityManager>,
    topic_name: &str,
    tenant: &str,
    client_id: &str,
    source_ip: &str,
    action: &EnumAclAction,
) -> bool {
    if let Some(tenant_map) = security_manager.security_metadata.acl_client_id.get(tenant) {
        if let Some(acl_list) = tenant_map.get(client_id) {
            return check_for_deny(&acl_list, action, topic_name, source_ip);
        }
    }
    false
}

fn check_for_deny(
    acl_list: &[SecurityAcl],
    action: &EnumAclAction,
    topic_name: &str,
    source_ip: &str,
) -> bool {
    for acl in acl_list.iter() {
        if acl.permission != EnumAclPermission::Deny {
            continue;
        }
        if acl.action != *action && acl.action != EnumAclAction::All {
            continue;
        }

        if topic_match(topic_name, &acl.topic) && ip_match(source_ip, &acl.ip) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{is_user_acl_deny, normalize_source_ip};
    use crate::manager::SecurityManager;
    use metadata_struct::auth::acl::{
        EnumAclAction, EnumAclPermission, EnumAclResourceType, SecurityAcl,
    };
    use std::sync::Arc;

    fn make_acl(
        tenant: &str,
        user: &str,
        action: EnumAclAction,
        permission: EnumAclPermission,
    ) -> SecurityAcl {
        SecurityAcl {
            name: "test".to_string(),
            desc: String::new(),
            tenant: tenant.to_string(),
            resource_type: EnumAclResourceType::User,
            resource_name: user.to_string(),
            topic: "sensor/data".to_string(),
            ip: "*".to_string(),
            action,
            permission,
        }
    }

    #[test]
    fn test_normalize_source_ip() {
        assert_eq!(normalize_source_ip("192.168.1.1:12345"), "192.168.1.1");
        assert_eq!(normalize_source_ip("[::1]:8080"), "::1");
        assert_eq!(normalize_source_ip("10.0.0.1"), "10.0.0.1");
    }

    #[test]
    fn test_is_user_acl_deny() {
        let sm = Arc::new(SecurityManager::new());
        let tenant = "t1";
        let user = "user1";

        sm.security_metadata
            .add_acl(make_acl(tenant, user, EnumAclAction::Publish, EnumAclPermission::Deny));

        assert!(is_user_acl_deny(&sm, "sensor/data", tenant, user, "1.2.3.4", &EnumAclAction::Publish));

        assert!(!is_user_acl_deny(&sm, "sensor/data", tenant, user, "1.2.3.4", &EnumAclAction::Subscribe));

        assert!(!is_user_acl_deny(&sm, "sensor/data", tenant, "other_user", "1.2.3.4", &EnumAclAction::Publish));
    }
}
