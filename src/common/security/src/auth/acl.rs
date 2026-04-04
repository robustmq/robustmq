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
use common_base::error::common::CommonError;
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
) -> Result<bool, CommonError> {
    if let Some(tenant_map) = security_manager.metadata.acl_user.get(tenant) {
        if let Some(acl_list) = tenant_map.get(user) {
            return check_acl_rules(&acl_list, action, topic_name, source_ip);
        }
    }
    Ok(false)
}

pub fn is_client_id_acl_deny(
    security_manager: &Arc<SecurityManager>,
    topic_name: &str,
    tenant: &str,
    client_id: &str,
    source_ip: &str,
    action: &EnumAclAction,
) -> Result<bool, CommonError> {
    if let Some(tenant_map) = security_manager.metadata.acl_client_id.get(tenant) {
        if let Some(acl_list) = tenant_map.get(client_id) {
            return check_acl_rules(&acl_list, action, topic_name, source_ip);
        }
    }
    Ok(false)
}

fn check_acl_rules(
    acl_list: &[SecurityAcl],
    action: &EnumAclAction,
    topic_name: &str,
    source_ip: &str,
) -> Result<bool, CommonError> {
    for acl in acl_list.iter() {
        if acl.action != *action && acl.action != EnumAclAction::All {
            continue;
        }
        if !topic_match(topic_name, &acl.topic) || !ip_match(source_ip, &acl.ip)? {
            continue;
        }
        return Ok(acl.permission == EnumAclPermission::Deny);
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::{is_client_id_acl_deny, is_user_acl_deny, normalize_source_ip};
    use crate::manager::SecurityManager;
    use metadata_struct::auth::acl::{
        EnumAclAction, EnumAclPermission, EnumAclResourceType, SecurityAcl,
    };
    use std::sync::Arc;

    fn make_acl(
        tenant: &str,
        user: &str,
        topic: &str,
        action: EnumAclAction,
        permission: EnumAclPermission,
    ) -> SecurityAcl {
        SecurityAcl {
            name: format!("{}-{:?}-{:?}", user, action, permission),
            desc: String::new(),
            tenant: tenant.to_string(),
            resource_type: EnumAclResourceType::User,
            resource_name: user.to_string(),
            topic: topic.to_string(),
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

        // Deny rule: blocked
        sm.metadata.add_acl(make_acl(
            tenant,
            user,
            "sensor/data",
            EnumAclAction::Publish,
            EnumAclPermission::Deny,
        ));
        assert!(is_user_acl_deny(
            &sm,
            "sensor/data",
            tenant,
            user,
            "1.2.3.4",
            &EnumAclAction::Publish
        )
        .unwrap());

        // Different action: no match → not denied
        assert!(!is_user_acl_deny(
            &sm,
            "sensor/data",
            tenant,
            user,
            "1.2.3.4",
            &EnumAclAction::Subscribe
        )
        .unwrap());

        // Unknown user: no rules → not denied
        assert!(!is_user_acl_deny(
            &sm,
            "sensor/data",
            tenant,
            "other_user",
            "1.2.3.4",
            &EnumAclAction::Publish
        )
        .unwrap());

        // Allow rule wins over no prior deny: first-match allow → not denied
        let sm2 = Arc::new(SecurityManager::new());
        sm2.metadata.add_acl(make_acl(
            tenant,
            user,
            "sensor/data",
            EnumAclAction::Publish,
            EnumAclPermission::Allow,
        ));
        assert!(!is_user_acl_deny(
            &sm2,
            "sensor/data",
            tenant,
            user,
            "1.2.3.4",
            &EnumAclAction::Publish
        )
        .unwrap());

        // Allow on specific topic, Deny on wildcard: first-match (Allow) wins
        let sm3 = Arc::new(SecurityManager::new());
        sm3.metadata.add_acl(make_acl(
            tenant,
            user,
            "sensor/data",
            EnumAclAction::Publish,
            EnumAclPermission::Allow,
        ));
        sm3.metadata.add_acl(make_acl(
            tenant,
            user,
            "*",
            EnumAclAction::Publish,
            EnumAclPermission::Deny,
        ));
        assert!(!is_user_acl_deny(
            &sm3,
            "sensor/data",
            tenant,
            user,
            "1.2.3.4",
            &EnumAclAction::Publish
        )
        .unwrap());
        assert!(is_user_acl_deny(
            &sm3,
            "other/topic",
            tenant,
            user,
            "1.2.3.4",
            &EnumAclAction::Publish
        )
        .unwrap());
    }

    #[test]
    fn test_is_client_id_acl_deny() {
        let sm = Arc::new(SecurityManager::new());
        let tenant = "t1";
        let client_id = "device-001";

        sm.metadata.add_acl(SecurityAcl {
            name: "acl-client".to_string(),
            desc: String::new(),
            tenant: tenant.to_string(),
            resource_type: EnumAclResourceType::ClientId,
            resource_name: client_id.to_string(),
            topic: "cmd/device".to_string(),
            ip: "*".to_string(),
            action: EnumAclAction::Subscribe,
            permission: EnumAclPermission::Deny,
        });

        assert!(is_client_id_acl_deny(
            &sm,
            "cmd/device",
            tenant,
            client_id,
            "1.2.3.4",
            &EnumAclAction::Subscribe
        )
        .unwrap());
        assert!(!is_client_id_acl_deny(
            &sm,
            "cmd/device",
            tenant,
            client_id,
            "1.2.3.4",
            &EnumAclAction::Publish
        )
        .unwrap());
        assert!(!is_client_id_acl_deny(
            &sm,
            "cmd/device",
            tenant,
            "other-device",
            "1.2.3.4",
            &EnumAclAction::Subscribe
        )
        .unwrap());
    }

    #[test]
    fn test_action_all_matches_any_action() {
        let sm = Arc::new(SecurityManager::new());
        let tenant = "t1";
        let user = "u1";

        sm.metadata.add_acl(make_acl(
            tenant,
            user,
            "data/#",
            EnumAclAction::All,
            EnumAclPermission::Deny,
        ));

        assert!(is_user_acl_deny(
            &sm,
            "data/#",
            tenant,
            user,
            "1.2.3.4",
            &EnumAclAction::Publish
        )
        .unwrap());
        assert!(is_user_acl_deny(
            &sm,
            "data/#",
            tenant,
            user,
            "1.2.3.4",
            &EnumAclAction::Subscribe
        )
        .unwrap());
        assert!(is_user_acl_deny(
            &sm,
            "data/#",
            tenant,
            user,
            "1.2.3.4",
            &EnumAclAction::Retain
        )
        .unwrap());
    }
}
