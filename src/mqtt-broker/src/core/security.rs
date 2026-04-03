use crate::core::cache::MQTTCacheManager;
use crate::core::{error::MqttBrokerError, tenant::try_decode_username};
use crate::subscribe::common::get_sub_topic_name_list;
use broker_core::cache::NodeCacheManager;
use common_base::error::common::CommonError;
use common_metrics::mqtt::auth::{record_mqtt_acl_failed, record_mqtt_acl_success};
use common_security::auth::blacklist::is_connection_blacklisted;
use common_security::auth::is_allow_acl;
use common_security::login::password::password_check_by_login;
use common_security::{login::LoginType, manager::SecurityManager};
use metadata_struct::auth::acl::EnumAclAction;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{ConnectProperties, Login, QoS, Subscribe};
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

pub async fn security_connect_check(
    security_manager: &Arc<SecurityManager>,
    client_id: &str,
    source_ip_addr: &str,
    login: &Option<Login>,
) -> bool {
    let login = login.clone().unwrap_or_default();
    is_connection_blacklisted(security_manager, client_id, source_ip_addr, &login.username)
        .unwrap_or(true)
}

pub async fn security_publish_acl_check(
    security_manager: &Arc<SecurityManager>,
    connection: &MQTTConnection,
    topic_name: &str,
    retain: bool,
    qos: QoS,
) -> Result<(), CommonError> {
    let user = connection.login_user.clone().unwrap_or_default();
    if !is_allow_acl(
        security_manager,
        topic_name,
        &connection.tenant,
        &user,
        &connection.source_ip_addr,
        &connection.client_id,
        EnumAclAction::Publish,
        retain,
        qos,
    ) {
        record_mqtt_acl_failed();
        return Err(CommonError::NotAclAuth(topic_name.to_string()));
    }
    record_mqtt_acl_success();

    Ok(())
}

pub async fn security_subscribe_check(
    cache_manager: &Arc<MQTTCacheManager>,
    security_manager: &Arc<SecurityManager>,
    connection: &MQTTConnection,
    subscribe: &Subscribe,
) -> bool {
    let user = connection.login_user.clone().unwrap_or_default();
    for filter in subscribe.filters.iter() {
        let topic_list = get_sub_topic_name_list(cache_manager, &filter.path).await;
        for topic_name in topic_list {
            if !is_allow_acl(
                security_manager,
                &topic_name,
                &connection.tenant,
                &user,
                &connection.source_ip_addr,
                &connection.client_id,
                EnumAclAction::Subscribe,
                false,
                filter.qos,
            ) {
                return false;
            }
        }
    }
    true
}
