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

use common_config::mqtt::config::BrokerMqttConfig;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::{
    Connect, ConnectProperties, ConnectReturnCode, LastWill, LastWillProperties, Login, MqttPacket,
    MqttProtocol, PubAckReason, PubRecReason, Publish, PublishProperties, QoS, Subscribe,
    SubscribeReasonCode, UnsubAckReason, Unsubscribe,
};
use std::cmp::min;
use std::sync::Arc;

use super::cache::CacheManager;
use super::content_type::{
    payload_format_indicator_check_by_lastwill, payload_format_indicator_check_by_publish,
};
use super::error::MqttBrokerError;
use super::flow_control::{is_qos_message, is_subscribe_rate_exceeded};
use super::response::{
    response_packet_mqtt_connect_fail, response_packet_mqtt_puback_fail,
    response_packet_mqtt_pubrec_fail, response_packet_mqtt_suback, response_packet_mqtt_unsuback,
};
use super::sub_exclusive::{allow_exclusive_subscribe, already_exclusive_subscribe};
use super::topic::topic_name_validator;
use crate::common::pkid_storage::pkid_exists;
use crate::security::AuthDriver;
use crate::subscribe::common::sub_path_validator;
use crate::subscribe::manager::SubscribeManager;

pub fn connect_validator(
    protocol: &MqttProtocol,
    cluster: &BrokerMqttConfig,
    connect: &Connect,
    connect_properties: &Option<ConnectProperties>,
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
    login: &Option<Login>,
) -> Option<MqttPacket> {
    if cluster.security.is_self_protection_status {
        return Some(response_packet_mqtt_connect_fail(
            protocol,
            ConnectReturnCode::ServerBusy,
            connect_properties,
            Some(MqttBrokerError::ClusterIsInSelfProtection.to_string()),
        ));
    }

    if !connect.client_id.is_empty() && !client_id_validator(&connect.client_id) {
        return Some(response_packet_mqtt_connect_fail(
            protocol,
            ConnectReturnCode::ClientIdentifierNotValid,
            connect_properties,
            None,
        ));
    }

    if let Some(login_info) = login {
        if !username_validator(&login_info.username) || !password_validator(&login_info.password) {
            return Some(response_packet_mqtt_connect_fail(
                protocol,
                ConnectReturnCode::BadUserNamePassword,
                connect_properties,
                None,
            ));
        }
    }

    if let Some(will) = last_will {
        if will.topic.is_empty() {
            return Some(response_packet_mqtt_connect_fail(
                protocol,
                ConnectReturnCode::TopicNameInvalid,
                connect_properties,
                None,
            ));
        }

        let topic_name = match String::from_utf8(will.topic.to_vec()) {
            Ok(da) => da,
            Err(e) => {
                return Some(response_packet_mqtt_connect_fail(
                    protocol,
                    ConnectReturnCode::TopicNameInvalid,
                    connect_properties,
                    Some(e.to_string()),
                ));
            }
        };

        match topic_name_validator(&topic_name) {
            Ok(()) => {}
            Err(e) => {
                response_packet_mqtt_connect_fail(
                    protocol,
                    ConnectReturnCode::TopicNameInvalid,
                    connect_properties,
                    Some(e.to_string()),
                );
            }
        }

        if will.message.is_empty() {
            return Some(response_packet_mqtt_connect_fail(
                protocol,
                ConnectReturnCode::PayloadFormatInvalid,
                connect_properties,
                None,
            ));
        }

        if !payload_format_indicator_check_by_lastwill(last_will, last_will_properties) {
            return Some(response_packet_mqtt_connect_fail(
                protocol,
                ConnectReturnCode::PayloadFormatInvalid,
                connect_properties,
                None,
            ));
        }

        let max_packet_size = connection_max_packet_size(connect_properties, cluster) as usize;
        if will.message.len() > max_packet_size {
            return Some(response_packet_mqtt_connect_fail(
                protocol,
                ConnectReturnCode::PacketTooLarge,
                connect_properties,
                None,
            ));
        }

        if let Some(will_properties) = last_will_properties {
            if let Some(payload_format) = will_properties.payload_format_indicator {
                if payload_format == 1
                    && std::str::from_utf8(will.message.to_vec().as_slice()).is_err()
                {
                    return Some(response_packet_mqtt_connect_fail(
                        protocol,
                        ConnectReturnCode::PayloadFormatInvalid,
                        connect_properties,
                        None,
                    ));
                }
            }
        }
    }
    None
}

pub async fn publish_validator(
    protocol: &MqttProtocol,
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    connection: &MQTTConnection,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> Option<MqttPacket> {
    let is_puback = publish.qos != QoS::ExactlyOnce;

    if publish.qos == QoS::ExactlyOnce {
        match pkid_exists(
            cache_manager,
            client_pool,
            &connection.client_id,
            publish.pkid,
        )
        .await
        {
            Ok(res) => {
                if res {
                    return Some(response_packet_mqtt_pubrec_fail(
                        protocol,
                        connection,
                        publish.pkid,
                        PubRecReason::PacketIdentifierInUse,
                        None,
                    ));
                }
            }
            Err(e) => {
                return Some(response_packet_mqtt_pubrec_fail(
                    protocol,
                    connection,
                    publish.pkid,
                    PubRecReason::UnspecifiedError,
                    Some(e.to_string()),
                ));
            }
        };
    }

    let cluster = cache_manager.get_cluster_config();

    let max_packet_size = min(
        cluster.mqtt_protocol_config.max_packet_size,
        connection.max_packet_size,
    ) as usize;
    if publish.payload.len() > max_packet_size {
        if is_puback {
            return Some(response_packet_mqtt_puback_fail(
                protocol,
                connection,
                publish.pkid,
                PubAckReason::PayloadFormatInvalid,
                Some(
                    MqttBrokerError::PacketLengthError(max_packet_size, publish.payload.len())
                        .to_string(),
                ),
            ));
        } else {
            return Some(response_packet_mqtt_pubrec_fail(
                protocol,
                connection,
                publish.pkid,
                PubRecReason::PayloadFormatInvalid,
                Some(
                    MqttBrokerError::PacketLengthError(max_packet_size, publish.payload.len())
                        .to_string(),
                ),
            ));
        }
    }

    if let Some(properties) = publish_properties {
        if let Some(payload_format) = properties.payload_format_indicator {
            if payload_format == 1
                && std::str::from_utf8(publish.payload.to_vec().as_slice()).is_err()
            {
                if is_puback {
                    return Some(response_packet_mqtt_puback_fail(
                        protocol,
                        connection,
                        publish.pkid,
                        PubAckReason::PayloadFormatInvalid,
                        Some(MqttBrokerError::PayloadFormatInvalid.to_string()),
                    ));
                } else {
                    return Some(response_packet_mqtt_pubrec_fail(
                        protocol,
                        connection,
                        publish.pkid,
                        PubRecReason::PayloadFormatInvalid,
                        Some(MqttBrokerError::PayloadFormatInvalid.to_string()),
                    ));
                }
            }
        }
    }

    if is_qos_message(publish.qos)
        && connection.get_recv_qos_message() >= cluster.mqtt_protocol_config.receive_max as isize
    {
        if is_puback {
            return Some(response_packet_mqtt_puback_fail(
                protocol,
                connection,
                publish.pkid,
                PubAckReason::QuotaExceeded,
                None,
            ));
        } else {
            return Some(response_packet_mqtt_pubrec_fail(
                protocol,
                connection,
                publish.pkid,
                PubRecReason::QuotaExceeded,
                None,
            ));
        }
    }

    if !payload_format_indicator_check_by_publish(publish, publish_properties) {
        if is_puback {
            return Some(response_packet_mqtt_puback_fail(
                protocol,
                connection,
                publish.pkid,
                PubAckReason::PayloadFormatInvalid,
                None,
            ));
        } else {
            return Some(response_packet_mqtt_pubrec_fail(
                protocol,
                connection,
                publish.pkid,
                PubRecReason::PayloadFormatInvalid,
                None,
            ));
        }
    }

    if let Some(properties) = publish_properties {
        if let Some(alias) = properties.topic_alias {
            let cluster = cache_manager.get_cluster_config();
            if alias > cluster.mqtt_protocol_config.topic_alias_max {
                if is_puback {
                    return Some(response_packet_mqtt_puback_fail(
                        protocol,
                        connection,
                        publish.pkid,
                        PubAckReason::UnspecifiedError,
                        Some(MqttBrokerError::TopicAliasTooLong(alias).to_string()),
                    ));
                } else {
                    return Some(response_packet_mqtt_pubrec_fail(
                        protocol,
                        connection,
                        publish.pkid,
                        PubRecReason::UnspecifiedError,
                        Some(MqttBrokerError::TopicAliasTooLong(alias).to_string()),
                    ));
                }
            }
        }
    }

    None
}

pub async fn subscribe_validator(
    protocol: &MqttProtocol,
    auth_driver: &Arc<AuthDriver>,
    metadata_cache: &Arc<CacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    connection: &MQTTConnection,
    subscribe: &Subscribe,
) -> Option<MqttPacket> {
    let mut return_codes: Vec<SubscribeReasonCode> = Vec::new();

    for filter in subscribe.filters.clone() {
        if sub_path_validator(&filter.path).is_err() {
            return_codes.push(SubscribeReasonCode::TopicFilterInvalid);
            continue;
        }
    }

    if !return_codes.is_empty() {
        return Some(response_packet_mqtt_suback(
            protocol,
            connection,
            subscribe.packet_identifier,
            return_codes,
            None,
        ));
    }

    if is_subscribe_rate_exceeded() {
        return Some(response_packet_mqtt_suback(
            protocol,
            connection,
            subscribe.packet_identifier,
            vec![SubscribeReasonCode::QuotaExceeded],
            None,
        ));
    }

    if !allow_exclusive_subscribe(metadata_cache, subscribe) {
        return Some(response_packet_mqtt_suback(
            protocol,
            connection,
            subscribe.packet_identifier,
            vec![SubscribeReasonCode::ExclusiveSubscriptionDisabled],
            None,
        ));
    }

    if already_exclusive_subscribe(subscribe_manager, subscribe) {
        return Some(response_packet_mqtt_suback(
            protocol,
            connection,
            subscribe.packet_identifier,
            vec![SubscribeReasonCode::TopicSubscribed],
            None,
        ));
    }

    if !auth_driver.allow_subscribe(connection, subscribe).await {
        return Some(response_packet_mqtt_suback(
            protocol,
            connection,
            subscribe.packet_identifier,
            vec![SubscribeReasonCode::NotAuthorized],
            None,
        ));
    }

    None
}

pub async fn un_subscribe_validator(
    client_id: &str,
    subscribe_manager: &Arc<SubscribeManager>,
    connection: &MQTTConnection,
    un_subscribe: &Unsubscribe,
) -> Option<MqttPacket> {
    let mut return_codes: Vec<UnsubAckReason> = Vec::new();
    for path in un_subscribe.filters.clone() {
        if sub_path_validator(&path).is_err() {
            return_codes.push(UnsubAckReason::TopicFilterInvalid);
            continue;
        }
    }
    if !return_codes.is_empty() {
        return Some(response_packet_mqtt_unsuback(
            connection,
            un_subscribe.pkid,
            return_codes,
            None,
        ));
    }

    for path in un_subscribe.filters.clone() {
        if subscribe_manager.get_subscribe(client_id, &path).is_none() {
            return Some(response_packet_mqtt_unsuback(
                connection,
                un_subscribe.pkid,
                vec![UnsubAckReason::NoSubscriptionExisted],
                Some(MqttBrokerError::SubscriptionPathNotExists(path).to_string()),
            ));
        }
    }

    None
}

pub fn is_request_problem_info(connect_properties: &Option<ConnectProperties>) -> bool {
    if let Some(properties) = connect_properties {
        if let Some(problem_info) = properties.request_problem_info {
            return problem_info == 1;
        }
    }
    false
}

pub fn connection_max_packet_size(
    connect_properties: &Option<ConnectProperties>,
    cluster: &BrokerMqttConfig,
) -> u32 {
    if let Some(properties) = connect_properties {
        if let Some(size) = properties.max_packet_size {
            return min(size, cluster.mqtt_protocol_config.max_packet_size);
        }
    }
    cluster.mqtt_protocol_config.max_packet_size
}

pub fn client_id_validator(client_id: &str) -> bool {
    if client_id.len() == 5 && client_id.len() > 23 {
        return false;
    }
    true
}

pub fn username_validator(username: &str) -> bool {
    if username.is_empty() {
        return false;
    }
    true
}

pub fn password_validator(password: &str) -> bool {
    if password.is_empty() {
        return false;
    }
    true
}

#[cfg(test)]
mod test {
    #[test]
    pub fn topic_name_validator_test() {}
}
