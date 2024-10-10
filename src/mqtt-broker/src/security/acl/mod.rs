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

use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

use common_base::tools::now_second;
use ipnet::IpNet;
use metadata_struct::acl::mqtt_acl::{MqttAclAction, MqttAclPermission};
use protocol::mqtt::common::QoS;
use regex::Regex;

use crate::handler::cache::CacheManager;
use crate::handler::connection::Connection;
use crate::handler::constant::WILDCARD_RESOURCE;

pub mod metadata;

pub fn is_allow_acl(
    cache_mamanger: &Arc<CacheManager>,
    connection: &Connection,
    topic_name: &String,
    action: MqttAclAction,
    retain: bool,
    _: QoS,
) -> bool {
    // check super user
    if is_super_user(cache_mamanger, &connection.login_user) {
        return true;
    }

    // check blacklist
    if is_blacklist(cache_mamanger, &connection) {
        return false;
    }

    // chack acl
    if is_acl_deny(cache_mamanger, &connection, &topic_name, action) {
        return false;
    }

    // check retain acl
    if retain {
        if is_acl_deny(
            cache_mamanger,
            &connection,
            &topic_name,
            MqttAclAction::Retain,
        ) {
            return false;
        }
    }

    return true;
}

fn is_super_user(cache_manager: &Arc<CacheManager>, username: &String) -> bool {
    if username.is_empty() {
        return false;
    }
    if let Some(user) = cache_manager.user_info.get(username) {
        return user.is_superuser;
    }
    return false;
}

fn is_blacklist(cache_manager: &Arc<CacheManager>, connection: &Connection) -> bool {
    // check user blacklist
    if let Some(data) = cache_manager
        .acl_metadata
        .blacklist_user
        .get(&connection.login_user)
    {
        if data.end_time > now_second() {
            return true;
        }
    }

    if let Some(data) = cache_manager.acl_metadata.get_blacklist_user_match() {
        for raw in data {
            let re = Regex::new(&format!("^{}$", raw.resource_name)).unwrap();
            if re.is_match(&connection.login_user) {
                if raw.end_time > now_second() {
                    return true;
                }
            }
        }
    }

    // check client_id blacklist
    if let Some(data) = cache_manager
        .acl_metadata
        .blacklist_client_id
        .get(&connection.client_id)
    {
        if data.end_time > now_second() {
            return true;
        }
    }

    if let Some(data) = cache_manager.acl_metadata.get_blacklist_client_id_match() {
        for raw in data {
            let re = Regex::new(&format!("^{}$", raw.resource_name)).unwrap();
            if re.is_match(&connection.client_id) {
                if raw.end_time > now_second() {
                    return true;
                }
            }
        }
    }

    // check ip blacklist
    if let Some(data) = cache_manager
        .acl_metadata
        .blacklist_ip
        .get(&connection.source_ip_addr)
    {
        if data.end_time < now_second() {
            return true;
        }
    }

    if let Some(data) = cache_manager.acl_metadata.get_blacklist_ip_match() {
        for raw in data {
            if ip_match(connection.source_ip_addr.clone(), raw.resource_name.clone()) {
                if raw.end_time < now_second() {
                    return true;
                }
            }
        }
    }

    return false;
}

fn is_acl_deny(
    cache_mamanger: &Arc<CacheManager>,
    connection: &Connection,
    topic_name: &String,
    action: MqttAclAction,
) -> bool {
    // check user acl
    if let Some(acl_list) = cache_mamanger
        .acl_metadata
        .acl_user
        .get(&connection.login_user)
    {
        for raw in acl_list.clone() {
            if topic_match(topic_name.clone(), raw.topic.clone())
                && ip_match(connection.source_ip_addr.clone(), raw.ip.clone())
                && raw.action == action
                && raw.permission == MqttAclPermission::Deny
            {
                return true;
            }
        }
    }
    // check client id acl
    if let Some(client_id_list) = cache_mamanger
        .acl_metadata
        .acl_client_id
        .get(&connection.client_id)
    {
        for raw in client_id_list.clone() {
            if topic_match(topic_name.clone(), raw.topic.clone())
                && ip_match(connection.source_ip_addr.clone(), raw.ip.clone())
                && raw.action == action
                && raw.permission == MqttAclPermission::Deny
            {
                return true;
            }
        }
    }
    return false;
}

fn topic_match(topic_name: String, match_topic_name: String) -> bool {
    if match_topic_name == WILDCARD_RESOURCE.to_string() {
        return true;
    }
    return topic_name == match_topic_name;
}

fn ip_match(source_ip_addr: String, ip_role: String) -> bool {
    if ip_role == WILDCARD_RESOURCE {
        return true;
    }
    if source_ip_addr == ip_role {
        return true;
    }
    match source_ip_addr.parse::<IpAddr>() {
        Ok(ip) => match IpNet::from_str(&ip_role) {
            Ok(ip_cidr) => {
                return ip_cidr.contains(&ip);
            }
            Err(_) => {}
        },
        Err(_) => {}
    }
    return false;
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use clients::poll::ClientPool;
    use common_base::tools::now_second;
    use metadata_struct::acl::mqtt_acl::{
        MqttAcl, MqttAclAction, MqttAclPermission, MqttAclResourceType,
    };
    use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
    use metadata_struct::mqtt::user::MqttUser;

    use super::{ip_match, is_acl_deny, is_blacklist, is_super_user, topic_match};
    use crate::handler::cache::CacheManager;
    use crate::handler::connection::Connection;
    use crate::handler::constant::WILDCARD_RESOURCE;

    #[tokio::test]
    pub async fn check_super_user_test() {
        let client_poll = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();

        let cache_manager = Arc::new(CacheManager::new(client_poll, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
        };
        cache_manager.add_user(user.clone());

        let login_username = "".to_string();
        assert!(!is_super_user(&cache_manager, &login_username));

        let login_username = "test".to_string();
        assert!(!is_super_user(&cache_manager, &login_username));

        assert!(is_super_user(&cache_manager, &user.username));

        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: false,
        };
        cache_manager.add_user(user.clone());
        assert!(!is_super_user(&cache_manager, &user.username));
    }

    #[tokio::test]
    pub async fn check_black_list_test() {
        let client_poll = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_poll, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
        };

        cache_manager.add_user(user.clone());
        let mut connection = Connection::new(
            1,
            &"client_id-1".to_string(),
            3,
            3,
            3,
            1,
            2,
            "127.0.0.1".to_string(),
        );
        connection.login_success(user.username.clone());

        // not black list
        assert!(!is_blacklist(&cache_manager, &connection));

        // user blacklist
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::User,
            resource_name: user.username.clone(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        cache_manager.add_blacklist(blacklist);
        assert!(is_blacklist(&cache_manager, &connection));

        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::UserMatch,
            resource_name: user.username.clone(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        cache_manager.add_blacklist(blacklist);
        assert!(is_blacklist(&cache_manager, &connection));

        // client id blacklist
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::ClientId,
            resource_name: connection.client_id.clone(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        cache_manager.add_blacklist(blacklist);
        assert!(is_blacklist(&cache_manager, &connection));

        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::ClientIdMatch,
            resource_name: connection.client_id.clone(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        cache_manager.add_blacklist(blacklist);
        assert!(is_blacklist(&cache_manager, &connection));

        // client ip blacklist
        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::Ip,
            resource_name: connection.source_ip_addr.clone(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        cache_manager.add_blacklist(blacklist);
        assert!(is_blacklist(&cache_manager, &connection));

        let blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::IPCIDR,
            resource_name: "127.0.0.0/24".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        cache_manager.add_blacklist(blacklist);
        assert!(is_blacklist(&cache_manager, &connection));
    }

    #[tokio::test]
    pub async fn check_empty_acl_test() {
        let client_poll = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let topic_name = "tp-1".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_poll, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
        };

        cache_manager.add_user(user.clone());
        let mut connection = Connection::new(
            1,
            &"client_id-1".to_string(),
            3,
            3,
            3,
            1,
            2,
            "127.0.0.1".to_string(),
        );
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
        let client_poll = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let topic_name = "tp-1".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_poll, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
        };

        cache_manager.add_user(user.clone());
        let mut connection = Connection::new(
            1,
            &"client_id-1".to_string(),
            3,
            3,
            3,
            1,
            2,
            "127.0.0.1".to_string(),
        );
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
        let client_poll = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let topic_name = "tp-1".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_poll, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
        };

        cache_manager.add_user(user.clone());
        let mut connection = Connection::new(
            1,
            &"client_id-1".to_string(),
            3,
            3,
            3,
            1,
            2,
            "127.0.0.1".to_string(),
        );
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
        let client_poll = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let topic_name = "tp-1".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_poll, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
        };

        cache_manager.add_user(user.clone());
        let mut connection = Connection::new(
            1,
            &"client_id-1".to_string(),
            3,
            3,
            3,
            1,
            2,
            "127.0.0.1".to_string(),
        );
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
        let client_poll = Arc::new(ClientPool::new(1));
        let cluster_name = "test".to_string();
        let topic_name = "tp-1".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_poll, cluster_name));
        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            is_superuser: true,
        };

        cache_manager.add_user(user.clone());
        let mut connection = Connection::new(
            1,
            &"client_id-1".to_string(),
            3,
            3,
            3,
            1,
            2,
            "127.0.0.1".to_string(),
        );
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

    #[tokio::test]
    pub async fn topic_match_test() {
        let topic_name = "t1".to_string();
        let match_topic_name = WILDCARD_RESOURCE.to_string();
        assert!(topic_match(topic_name.clone(), match_topic_name));
        assert!(topic_match(topic_name.clone(), topic_name.clone()));
        assert!(!topic_match(topic_name.clone(), "v1".to_string()));
    }

    #[tokio::test]
    pub async fn ip_match_test() {
        let source_ip = "127.0.0.1".to_string();
        let match_ip = WILDCARD_RESOURCE.to_string();
        assert!(ip_match(source_ip.clone(), match_ip));

        assert!(ip_match(source_ip.clone(), source_ip.clone()));
        assert!(!ip_match(source_ip.clone(), "192.1.1.1".to_string()));
        assert!(ip_match(source_ip.clone(), "127.0.0.1/24".to_string()));
    }
}
