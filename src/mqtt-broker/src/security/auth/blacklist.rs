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
    handler::{cache::MQTTCacheManager, error::MqttBrokerError},
    security::auth::common::ip_match,
};
use common_base::tools::now_second;
use metadata_struct::mqtt::connection::MQTTConnection;
use regex::Regex;
use std::sync::Arc;
use tracing::info;

pub fn is_blacklist(
    cache_manager: &Arc<MQTTCacheManager>,
    connection: &MQTTConnection,
) -> Result<bool, MqttBrokerError> {
    // todo: I believe this code can be refactored using the Chain of Responsibility pattern.
    // check user blacklist
    if let Some(data) = cache_manager
        .acl_metadata
        .blacklist_user
        .get(&connection.login_user)
    {
        if data.end_time > now_second() {
            info!("user blacklist banned,user:{}", &connection.login_user);
            return Ok(true);
        }
    }

    if let Some(data) = cache_manager.acl_metadata.get_blacklist_user_match() {
        for raw in data {
            let re = Regex::new(&format!("^{}$", raw.resource_name))?;
            if re.is_match(&connection.login_user) && raw.end_time > now_second() {
                info!(
                    "user blacklist banned by match,user:{}",
                    &connection.login_user
                );
                return Ok(true);
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
            info!(
                "client_id blacklist banned, client_id:{}",
                &connection.client_id
            );
            return Ok(true);
        }
    }

    if let Some(data) = cache_manager.acl_metadata.get_blacklist_client_id_match() {
        for raw in data {
            let re = Regex::new(&format!("^{}$", raw.resource_name))?;
            if re.is_match(&connection.client_id) && raw.end_time > now_second() {
                info!(
                    "client_id blacklist banned by match,client_id:{}",
                    &connection.client_id
                );
                return Ok(true);
            }
        }
    }

    // check ip blacklist
    if let Some(data) = cache_manager
        .acl_metadata
        .blacklist_ip
        .get(&connection.source_ip_addr)
    {
        if data.end_time > now_second() {
            info!(
                "ip blacklist banned,source_ip_addr:{}",
                &connection.source_ip_addr
            );
            return Ok(true);
        }
    }

    if let Some(data) = cache_manager.acl_metadata.get_blacklist_ip_match() {
        for raw in data {
            if ip_match(&connection.source_ip_addr, &raw.resource_name)
                && raw.end_time > now_second()
            {
                info!(
                    "ip blacklist banned by match,source_ip_addr:{}",
                    &connection.source_ip_addr
                );
                return Ok(true);
            }
        }
    }

    Ok(false)
}

#[cfg(test)]
mod test {
    use super::is_blacklist;
    use crate::common::tool::test_build_mqtt_cache_manager;
    use crate::handler::cache::MQTTCacheManager;
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
    use common_base::tools::{local_hostname, now_second};
    use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
    use metadata_struct::mqtt::connection::{ConnectionConfig, MQTTConnection};
    use metadata_struct::mqtt::user::MqttUser;
    use std::sync::Arc;

    struct TestFixture {
        cache_manager: Arc<MQTTCacheManager>,
        connection: MQTTConnection,
        user: MqttUser,
    }

    async fn setup() -> TestFixture {
        let cache_manager = test_build_mqtt_cache_manager().await;

        let user = MqttUser {
            username: "loboxu".to_string(),
            password: "lobo_123".to_string(),
            salt: None,
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

        TestFixture {
            cache_manager,
            connection,
            user,
        }
    }

    fn assert_is_blacklisted(
        fixture: &TestFixture,
        blacklist_type: MqttAclBlackListType,
        resource_name: String,
    ) {
        let blacklist = MqttAclBlackList {
            blacklist_type,
            resource_name,
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        fixture.cache_manager.add_blacklist(blacklist);

        assert!(
            is_blacklist(&fixture.cache_manager, &fixture.connection).unwrap(),
            "Expected to be blacklisted but was not"
        );
    }

    #[tokio::test]
    async fn test_not_blacklist_default() {
        let fixture = setup().await;
        assert!(!is_blacklist(&fixture.cache_manager, &fixture.connection).unwrap_or(true))
    }

    #[tokio::test]
    async fn test_blacklist_client_id() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::ClientId,
            fixture.connection.client_id.clone(),
        );
    }

    #[tokio::test]
    async fn test_blacklist_user() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::User,
            fixture.user.username.clone(),
        );
    }

    #[tokio::test]
    async fn test_blacklist_ip() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::Ip,
            fixture.connection.source_ip_addr.clone(),
        );
    }

    #[tokio::test]
    async fn test_blacklist_client_id_match() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::ClientIdMatch,
            fixture.connection.client_id.clone(),
        );
    }

    #[tokio::test]
    async fn test_blacklist_user_match() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::UserMatch,
            "lobo.*".to_string(),
        );
    }

    #[tokio::test]
    async fn test_blacklist_ip_cidr() {
        let fixture = setup().await;
        assert_is_blacklisted(
            &fixture,
            MqttAclBlackListType::IPCIDR,
            "127.0.0.0/24".to_string(),
        );
    }
}
