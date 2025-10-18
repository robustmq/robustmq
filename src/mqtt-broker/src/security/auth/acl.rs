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
    handler::cache::MQTTCacheManager,
    security::auth::common::{ip_match, topic_match},
};
use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
use metadata_struct::{acl::mqtt_acl::MqttAcl, mqtt::connection::MQTTConnection};
use std::sync::Arc;

pub fn is_acl_deny(
    cache_manager: &Arc<MQTTCacheManager>,
    connection: &MQTTConnection,
    topic_name: &str,
    action: MqttAclAction,
) -> bool {
    // check user acl
    if let Some(acl_list) = cache_manager
        .acl_metadata
        .acl_user
        .get(&connection.login_user)
    {
        return check_for_deny(&acl_list, &action, topic_name, &connection.source_ip_addr);
    }

    // check client id acl
    if let Some(acl_list) = cache_manager
        .acl_metadata
        .acl_client_id
        .get(&connection.client_id)
    {
        return check_for_deny(&acl_list, &action, topic_name, &connection.source_ip_addr);
    }

    false
}

fn check_for_deny(
    acl_list: &[MqttAcl],
    action: &MqttAclAction,
    topic_name: &str,
    source_ip: &str,
) -> bool {
    for acl in acl_list.iter() {
        if topic_match(topic_name, &acl.topic)
            && ip_match(source_ip, &acl.ip)
            && (acl.action == *action || acl.action == MqttAclAction::All)
            && acl.permission == MqttAclPermission::Deny
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod test {
    use super::is_acl_deny;
    use crate::common::tool::test_build_mqtt_cache_manager;
    use crate::handler::cache::MQTTCacheManager;
    use crate::handler::constant::WILDCARD_RESOURCE;
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
            client_id: "client_id-1".to_string(),
            receive_maximum: 3,
            max_packet_size: 3,
            topic_alias_max: 3,
            request_problem_info: 1,
            keep_alive: 2,
            source_ip_addr: local_hostname(),
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
    pub async fn check_empty_acl_test() {
        let fixture = setup().await;
        assert!(!is_acl_deny(
            &fixture.cache_manager,
            &fixture.connection,
            &fixture.topic_name,
            MqttAclAction::Publish,
        ));

        assert!(!is_acl_deny(
            &fixture.cache_manager,
            &fixture.connection,
            &fixture.topic_name,
            MqttAclAction::Subscribe
        ));
    }

    #[tokio::test]
    async fn test_user_is_denied_by_specific_topic_rule() {
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
        ),);

        assert!(!is_acl_deny(
            &fixture.cache_manager,
            &fixture.connection,
            &fixture.topic_name,
            MqttAclAction::Subscribe
        ),);
    }

    #[tokio::test]
    async fn test_clientid_is_denied_by_wildcard_topic_rule() {
        let fixture = setup().await;
        add_deny_rule(
            &fixture,
            MqttAclResourceType::ClientId,
            WILDCARD_RESOURCE,
            MqttAclAction::All,
        );

        assert!(is_acl_deny(
            &fixture.cache_manager,
            &fixture.connection,
            &fixture.topic_name,
            MqttAclAction::Publish
        ),);

        assert!(is_acl_deny(
            &fixture.cache_manager,
            &fixture.connection,
            "tp-2",
            MqttAclAction::Subscribe
        ),);
    }

    mod check_for_deny_tests {
        use crate::security::auth::acl::check_for_deny;

        use super::*;

        #[test]
        fn returns_false_for_empty_rule_list() {
            let rules: Vec<MqttAcl> = vec![];
            assert!(!check_for_deny(
                &rules,
                &MqttAclAction::Publish,
                "test/topic",
                "127.0.0.1"
            ));
        }

        #[test]
        fn returns_true_for_matching_deny_rule() {
            let rules = vec![MqttAcl {
                permission: MqttAclPermission::Deny,
                action: MqttAclAction::Publish,
                topic: "test/topic".to_string(),
                ip: "127.0.0.1".to_string(),
                resource_type: MqttAclResourceType::User,
                resource_name: "resource_name".to_string(),
            }];
            assert!(check_for_deny(
                &rules,
                &MqttAclAction::Publish,
                "test/topic",
                "127.0.0.1"
            ),);
        }

        #[test]
        fn returns_false_for_matching_allow_rule() {
            let rules = vec![MqttAcl {
                permission: MqttAclPermission::Allow,
                action: MqttAclAction::Publish,
                topic: "test/topic".to_string(),
                ip: "127.0.0.1".to_string(),

                resource_type: MqttAclResourceType::User,
                resource_name: "resource_name".to_string(),
            }];

            assert!(!check_for_deny(
                &rules,
                &MqttAclAction::Publish,
                "test/topic",
                "127.0.0.1"
            ),);
        }

        #[test]
        fn returns_false_when_action_does_not_match() {
            let rules = vec![MqttAcl {
                permission: MqttAclPermission::Deny,
                action: MqttAclAction::Subscribe,
                topic: "test/topic".to_string(),
                ip: "127.0.0.1".to_string(),

                resource_type: MqttAclResourceType::User,
                resource_name: "resource_name".to_string(),
            }];

            assert!(!check_for_deny(
                &rules,
                &MqttAclAction::Publish,
                "test/topic",
                "127.0.0.1"
            ),);
        }

        #[test]
        fn returns_true_when_rule_action_is_all() {
            let rules = vec![MqttAcl {
                permission: MqttAclPermission::Deny,
                action: MqttAclAction::All,
                topic: "test/topic".to_string(),
                ip: "127.0.0.1".to_string(),

                resource_type: MqttAclResourceType::User,
                resource_name: "resource_name".to_string(),
            }];
            assert!(check_for_deny(
                &rules,
                &MqttAclAction::Publish,
                "test/topic",
                "127.0.0.1"
            ),);
            assert!(check_for_deny(
                &rules,
                &MqttAclAction::Subscribe,
                "test/topic",
                "127.0.0.1"
            ),);
        }
    }
}
