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

use crate::core::cache::MQTTCacheManager;
use crate::core::{error::MqttBrokerError, tenant::try_decode_username};
use crate::subscribe::common::get_sub_topic_name_list;
use broker_core::cache::NodeCacheManager;
use common_base::error::common::CommonError;
use common_metrics::mqtt::auth::{record_mqtt_acl_failed, record_mqtt_acl_success};
use common_security::auth::acl::{is_client_id_acl_deny, is_user_acl_deny};
use common_security::auth::blacklist::{
    is_client_id_blacklisted, is_ip_blacklisted, is_user_blacklisted,
};
use common_security::login::password::password_check_by_login;
use common_security::login::super_user::is_super_user;
use common_security::{login::LoginType, manager::SecurityManager};
use metadata_struct::auth::acl::EnumAclAction;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{ConnectProperties, Login, Subscribe};
use std::str::FromStr;
use std::sync::Arc;

pub async fn security_login_check(
    security_manager: &Arc<SecurityManager>,
    node_cache: &Arc<NodeCacheManager>,
    login: &Option<Login>,
    _connect_properties: &Option<ConnectProperties>,
) -> Result<bool, MqttBrokerError> {
    let cluster = node_cache.get_cluster_config();

    if cluster.mqtt_runtime.secret_free_login {
        return Ok(true);
    }

    if let Some((_, authn)) = security_manager
        .authn_list_with_default()
        .into_iter()
        .next()
    {
        let login_type = LoginType::from_str(&authn.authn_type)
            .map_err(|_| CommonError::UnsupportedAuthType(authn.authn_type.clone()))?;

        return match login_type {
            LoginType::PasswordBased => {
                if let Some(user_info) = login {
                    let username = try_decode_username(&user_info.username);
                    let password = user_info.password.clone();
                    Ok(password_check_by_login(
                        security_manager,
                        &username,
                        &password,
                    ))
                } else {
                    Ok(false)
                }
            }
            LoginType::Jwt => Ok(false),
        };
    }

    Ok(false)
}

pub async fn security_is_allow_connect(
    security_manager: &Arc<SecurityManager>,
    tenant: &str,
    client_id: &str,
    source_ip: &str,
    login: &Option<Login>,
) -> bool {
    let login = login.clone().unwrap_or_default();

    !is_user_blacklisted(security_manager, tenant, &login.username)
        && !is_client_id_blacklisted(security_manager, tenant, client_id)
        && !is_ip_blacklisted(security_manager, tenant, source_ip)
}

pub async fn security_is_allow_publish(
    security_manager: &Arc<SecurityManager>,
    connection: &MQTTConnection,
    topic_name: &str,
    retain: bool,
) -> bool {
    let user = connection.login_user.clone().unwrap_or_default();
    if is_super_user(security_manager, &user) {
        record_mqtt_acl_success();
        return true;
    }

    let source_ip = connection.source_ip.as_str();

    // Message auth
    if is_client_id_acl_deny(
        security_manager,
        topic_name,
        &connection.tenant,
        &connection.client_id,
        source_ip,
        &EnumAclAction::Publish,
    ) {
        record_mqtt_acl_failed();
        return false;
    }

    if is_user_acl_deny(
        security_manager,
        topic_name,
        &connection.tenant,
        &user,
        source_ip,
        &EnumAclAction::Publish,
    ) {
        record_mqtt_acl_failed();
        return false;
    }

    // Retain auth
    if retain {
        if is_client_id_acl_deny(
            security_manager,
            topic_name,
            &connection.tenant,
            &connection.client_id,
            source_ip,
            &EnumAclAction::Retain,
        ) {
            record_mqtt_acl_failed();
            return false;
        }

        if is_user_acl_deny(
            security_manager,
            topic_name,
            &connection.tenant,
            &user,
            source_ip,
            &EnumAclAction::Retain,
        ) {
            record_mqtt_acl_failed();
            return false;
        }
    }

    record_mqtt_acl_success();
    true
}

pub async fn security_is_allow_subscribe(
    cache_manager: &Arc<MQTTCacheManager>,
    security_manager: &Arc<SecurityManager>,
    connection: &MQTTConnection,
    subscribe: &Subscribe,
) -> bool {
    let user = connection.login_user.clone().unwrap_or_default();
    let source_ip = connection.source_ip.as_str();

    for filter in subscribe.filters.iter() {
        let topic_list = get_sub_topic_name_list(cache_manager, &filter.path).await;
        for topic_name in topic_list {
            if is_client_id_acl_deny(
                security_manager,
                &topic_name,
                &connection.tenant,
                &connection.client_id,
                source_ip,
                &EnumAclAction::Subscribe,
            ) {
                record_mqtt_acl_failed();
                return false;
            }

            if is_user_acl_deny(
                security_manager,
                &topic_name,
                &connection.tenant,
                &user,
                source_ip,
                &EnumAclAction::Subscribe,
            ) {
                record_mqtt_acl_failed();
                return false;
            }
        }
    }

    true
}
