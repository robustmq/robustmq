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
    core::cache::MQTTCacheManager,
    security::auth::common::{ip_match, topic_match},
};
use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use metadata_struct::{acl::mqtt_acl::MqttAcl, mqtt::connection::MQTTConnection};
use std::{net::SocketAddr, sync::Arc};

fn normalize_source_ip(source_ip_addr: &str) -> String {
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

pub fn is_acl_deny(
    cache_manager: &Arc<MQTTCacheManager>,
    connection: &MQTTConnection,
    topic_name: &str,
    action: MqttAclAction,
) -> bool {
    let source_ip = normalize_source_ip(&connection.source_ip_addr);
    let user = connection.login_user.clone().unwrap_or_default();
    let user_deny = cache_manager
        .acl_metadata
        .acl_user
        .get(&user)
        .map(|acl_list| check_for_deny(&acl_list, &action, topic_name, &source_ip))
        .unwrap_or(false);

    let client_id_deny = cache_manager
        .acl_metadata
        .acl_client_id
        .get(&connection.client_id)
        .map(|acl_list| check_for_deny(&acl_list, &action, topic_name, &source_ip))
        .unwrap_or(false);

    user_deny || client_id_deny
}

fn check_for_deny(
    acl_list: &[MqttAcl],
    action: &MqttAclAction,
    topic_name: &str,
    source_ip: &str,
) -> bool {
    for acl in acl_list.iter() {
        if acl.permission != MqttAclPermission::Deny {
            continue;
        }
        if acl.action != *action && acl.action != MqttAclAction::All {
            continue;
        }
        if topic_match(topic_name, &acl.topic) && ip_match(source_ip, &acl.ip) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod test {
    use super::is_acl_deny;
    use crate::core::cache::MQTTCacheManager;
    use crate::core::constant::WILDCARD_RESOURCE;
    use crate::core::tool::test_build_mqtt_cache_manager;
    use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
    use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
    use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
    use common_base::tools::{local_hostname, now_second};
    use metadata_struct::acl::mqtt_acl::MqttAcl;
    use metadata_struct::mqtt::connection::{ConnectionConfig, MQTTConnection};
    use metadata_struct::mqtt::user::MqttUser;
    use std::sync::Arc;

    struct TestFixture {
        cache_manager: Arc<MQTTCacheManager>,
        connection: MQTTConnection,
        user: MqttUser,
        topic_name: String,
    }

    async fn setup() -> TestFixture {
        let topic_name = "tp-1".to_string();
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
            tenant: "tenant1".to_string(),
            client_id: "client_id-1".to_string(),
            receive_maximum: 3,
            max_packet_size: 3,
            topic_alias_max: 3,
            request_problem_info: 1,
            keep_alive: 2,
            source_ip_addr: local_hostname(),
            clean_session: true,
        };
        let mut connection = MQTTConnection::new(config);
        connection.login_success(user.username.clone());

        TestFixture {
            cache_manager,
            connection,
            user,
            topic_name,
        }
    }

    fn add_deny_rule(
        fixture: &TestFixture,
        resource_type: MqttAclResourceType,
        topic: &str,
        action: MqttAclAction,
    ) {
        let resource_name = match resource_type {
            MqttAclResourceType::User => fixture.user.username.clone(),
            MqttAclResourceType::ClientId => fixture.connection.client_id.clone(),
        };

        let acl = MqttAcl {
            resource_type,
            resource_name,
            topic: topic.to_string(),
            ip: WILDCARD_RESOURCE.to_string(),
            action,
            permission: MqttAclPermission::Deny,
        };
        fixture.cache_manager.add_acl(acl);
    }

    #[tokio::test]
    async fn empty_acl_returns_false() {
        let fixture = setup().await;
        assert!(!is_acl_deny(
            &fixture.cache_manager,
            &fixture.connection,
            &fixture.topic_name,
            MqttAclAction::Publish,
        ));
    }

    #[tokio::test]
    async fn user_specific_rule_blocks_publish_only() {
        let fixture = setup().await;
        add_deny_rule(
            &fixture,
            MqttAclResourceType::User,
            &fixture.topic_name,
            MqttAclAction::Publish,
        );

        assert!(is_acl_deny(
            &fixture.cache_manager,
            &fixture.connection,
            &fixture.topic_name,
            MqttAclAction::Publish
        ));

        assert!(!is_acl_deny(
            &fixture.cache_manager,
            &fixture.connection,
            &fixture.topic_name,
            MqttAclAction::Subscribe
        ));
    }

    #[tokio::test]
    async fn client_id_deny_still_effective_when_user_rules_exist() {
        let fixture = setup().await;

        fixture.cache_manager.add_acl(MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: fixture.user.username.clone(),
            topic: "other/topic".to_string(),
            ip: WILDCARD_RESOURCE.to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        });

        fixture.cache_manager.add_acl(MqttAcl {
            resource_type: MqttAclResourceType::ClientId,
            resource_name: fixture.connection.client_id.clone(),
            topic: WILDCARD_RESOURCE.to_string(),
            ip: WILDCARD_RESOURCE.to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Deny,
        });

        assert!(is_acl_deny(
            &fixture.cache_manager,
            &fixture.connection,
            &fixture.topic_name,
            MqttAclAction::Publish
        ));

        assert!(is_acl_deny(
            &fixture.cache_manager,
            &fixture.connection,
            "any/topic",
            MqttAclAction::Subscribe
        ));
    }

    #[tokio::test]
    async fn source_ip_with_port_is_normalized_for_acl_match() {
        let fixture = setup().await;
        let mut conn = fixture.connection.clone();
        conn.source_ip_addr = "127.0.0.1:1883".to_string();

        fixture.cache_manager.add_acl(MqttAcl {
            resource_type: MqttAclResourceType::ClientId,
            resource_name: conn.client_id.clone(),
            topic: fixture.topic_name.clone(),
            ip: "127.0.0.1".to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        });

        assert!(is_acl_deny(
            &fixture.cache_manager,
            &conn,
            &fixture.topic_name,
            MqttAclAction::Publish
        ));
    }

    mod check_for_deny_tests {
        use crate::security::auth::acl::check_for_deny;

        use super::*;

        #[test]
        fn check_for_deny_core_cases() {
            let rule = |permission: MqttAclPermission, action: MqttAclAction| MqttAcl {
                permission,
                action,
                topic: "test/topic".to_string(),
                ip: "127.0.0.1".to_string(),
                resource_type: MqttAclResourceType::User,
                resource_name: "resource_name".to_string(),
            };

            let cases = vec![
                (vec![], MqttAclAction::Publish, false),
                (
                    vec![rule(MqttAclPermission::Deny, MqttAclAction::Publish)],
                    MqttAclAction::Publish,
                    true,
                ),
                (
                    vec![rule(MqttAclPermission::Allow, MqttAclAction::Publish)],
                    MqttAclAction::Publish,
                    false,
                ),
                (
                    vec![rule(MqttAclPermission::Deny, MqttAclAction::Subscribe)],
                    MqttAclAction::Publish,
                    false,
                ),
                (
                    vec![rule(MqttAclPermission::Deny, MqttAclAction::All)],
                    MqttAclAction::Subscribe,
                    true,
                ),
            ];

            for (rules, action, expected) in cases {
                assert_eq!(
                    check_for_deny(&rules, &action, "test/topic", "127.0.0.1"),
                    expected
                );
            }
        }
    }
}
