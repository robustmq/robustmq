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
    handler::cache::CacheManager,
    security::auth::common::{ip_match, topic_match},
};
use metadata_struct::{
    acl::mqtt_acl::{MqttAclAction, MqttAclPermission},
    mqtt::connection::MQTTConnection,
};
use std::sync::Arc;

pub fn is_acl_deny(
    cache_manager: &Arc<CacheManager>,
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
        for raw in acl_list.clone() {
            if topic_match(topic_name, &raw.topic)
                && ip_match(&connection.source_ip_addr, &raw.ip)
                && (raw.action == action || raw.action == MqttAclAction::All)
                && raw.permission == MqttAclPermission::Deny
            {
                return true;
            }
        }
    }

    // check client id acl
    if let Some(client_id_list) = cache_manager
        .acl_metadata
        .acl_client_id
        .get(&connection.client_id)
    {
        for raw in client_id_list.clone() {
            if topic_match(topic_name, &raw.topic)
                && ip_match(&connection.source_ip_addr, &raw.ip)
                && (raw.action == action || raw.action == MqttAclAction::All)
                && raw.permission == MqttAclPermission::Deny
            {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod test {
    use super::is_acl_deny;
    use crate::handler::cache::CacheManager;
    use crate::handler::constant::WILDCARD_RESOURCE;
    use common_base::tools::local_hostname;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::acl::mqtt_acl::{
        MqttAcl, MqttAclAction, MqttAclPermission, MqttAclResourceType,
    };
    use metadata_struct::mqtt::connection::{ConnectionConfig, MQTTConnection};
    use metadata_struct::mqtt::user::MqttUser;
    use std::sync::Arc;

    #[tokio::test]
    pub async fn check_empty_acl_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let topic_name = "tp-1".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_pool, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
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

        assert!(!is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Publish
        ));

        assert!(!is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Subscribe
        ));
    }

    #[tokio::test]
    pub async fn check_user_wildcard_acl_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let topic_name = "tp-1".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_pool, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
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

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: user.username.clone(),
            topic: WILDCARD_RESOURCE.to_string(),
            ip: WILDCARD_RESOURCE.to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        };
        cache_manager.add_acl(acl);
        assert!(is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Publish
        ));

        assert!(!is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Subscribe
        ));

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: user.username.clone(),
            topic: WILDCARD_RESOURCE.to_string(),
            ip: WILDCARD_RESOURCE.to_string(),
            action: MqttAclAction::Subscribe,
            permission: MqttAclPermission::Deny,
        };
        cache_manager.add_acl(acl);
        assert!(is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Subscribe
        ));
    }

    #[tokio::test]
    pub async fn check_user_match_acl_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let topic_name = "tp-1".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_pool, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
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

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: user.username.clone(),
            topic: topic_name.clone(),
            ip: WILDCARD_RESOURCE.to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        };
        cache_manager.add_acl(acl);
        assert!(is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Publish
        ));

        assert!(!is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Subscribe
        ));

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: user.username.clone(),
            topic: topic_name.clone(),
            ip: WILDCARD_RESOURCE.to_string(),
            action: MqttAclAction::Subscribe,
            permission: MqttAclPermission::Deny,
        };
        cache_manager.add_acl(acl);
        assert!(is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Subscribe
        ));
    }

    #[tokio::test]
    pub async fn check_client_id_wildcard_acl_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let topic_name = "tp-1".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_pool, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
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

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::ClientId,
            resource_name: connection.client_id.clone(),
            topic: WILDCARD_RESOURCE.to_string(),
            ip: WILDCARD_RESOURCE.to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        };
        cache_manager.add_acl(acl);
        assert!(is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Publish
        ));

        assert!(!is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Subscribe
        ));

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::ClientId,
            resource_name: connection.client_id.clone(),
            topic: WILDCARD_RESOURCE.to_string(),
            ip: WILDCARD_RESOURCE.to_string(),
            action: MqttAclAction::Subscribe,
            permission: MqttAclPermission::Deny,
        };
        cache_manager.add_acl(acl);
        assert!(is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Subscribe
        ));
    }

    #[tokio::test]
    pub async fn check_client_id_match_acl_test() {
        let client_pool = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let topic_name = "tp-1".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_pool, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
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

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::ClientId,
            resource_name: connection.client_id.clone(),
            topic: topic_name.clone(),
            ip: WILDCARD_RESOURCE.to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        };
        cache_manager.add_acl(acl);
        assert!(is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Publish
        ));

        assert!(!is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Subscribe
        ));

        let acl = MqttAcl {
            resource_type: MqttAclResourceType::ClientId,
            resource_name: connection.client_id.clone(),
            topic: topic_name.clone(),
            ip: WILDCARD_RESOURCE.to_string(),
            action: MqttAclAction::Subscribe,
            permission: MqttAclPermission::Deny,
        };
        cache_manager.add_acl(acl);
        assert!(is_acl_deny(
            &cache_manager,
            &connection,
            &topic_name,
            MqttAclAction::Subscribe
        ));
    }
}
