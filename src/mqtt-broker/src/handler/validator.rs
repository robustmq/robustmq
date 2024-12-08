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

use std::cmp::min;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::SinkExt;
use grpc_clients::pool::ClientPool;
use log::error;
use metadata_struct::mqtt::cluster::MqttClusterDynamicConfig;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::{
    Connect, ConnectProperties, ConnectReturnCode, DisconnectReasonCode, LastWill,
    LastWillProperties, Login, MqttPacket, MqttProtocol, PubAckReason, PubRecReason, Publish,
    PublishProperties, QoS, Subscribe, SubscribeReasonCode, UnsubAckReason, Unsubscribe,
};
use tokio_util::codec::FramedWrite;

use super::cache::CacheManager;
use super::error::MqttBrokerError;
use super::flow_control::{
    is_connection_rate_exceeded, is_flow_control, is_subscribe_rate_exceeded,
};
use super::pkid::pkid_exists;
use super::response::{
    response_packet_mqtt_connect_fail, response_packet_mqtt_distinct_by_reason,
    response_packet_mqtt_puback_fail, response_packet_mqtt_pubrec_fail,
    response_packet_mqtt_suback, response_packet_mqtt_unsuback,
};
use super::topic::topic_name_validator;
use crate::security::authentication_acl;
use crate::security::login::is_ip_blacklist;
use crate::server::connection_manager::ConnectionManager;
use crate::subscribe::sub_common::sub_path_validator;

pub async fn tcp_establish_connection_check(
    addr: &SocketAddr,
    connection_manager: &Arc<ConnectionManager>,
    write_frame_stream: &mut FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, MqttCodec>,
) -> bool {
    if connection_manager.tcp_connect_num_check() {
        let packet_wrapper = MqttPacketWrapper {
            protocol_version: MqttProtocol::Mqtt5.into(),
            packet: response_packet_mqtt_distinct_by_reason(
                &MqttProtocol::Mqtt5,
                Some(DisconnectReasonCode::QuotaExceeded),
            ),
        };
        match write_frame_stream.send(packet_wrapper).await {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        }

        match write_frame_stream.close().await {
            Ok(_) => {
                error!(
                    "tcp connection failed to establish from IP: {}",
                    addr.to_string()
                );
            }
            Err(e) => error!("{}", e),
        }
        return false;
    }

    if is_connection_rate_exceeded() {
        let packet_wrapper = MqttPacketWrapper {
            protocol_version: MqttProtocol::Mqtt5.into(),
            packet: response_packet_mqtt_distinct_by_reason(
                &MqttProtocol::Mqtt5,
                Some(DisconnectReasonCode::ConnectionRateExceeded),
            ),
        };
        match write_frame_stream.send(packet_wrapper).await {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        }

        match write_frame_stream.close().await {
            Ok(_) => {
                error!(
                    "tcp connection failed to establish from IP: {}",
                    addr.to_string()
                );
            }
            Err(e) => error!("{}", e),
        }
        return false;
    }
    true
}

pub async fn tcp_tls_establish_connection_check(
    addr: &SocketAddr,
    connection_manager: &Arc<ConnectionManager>,
    write_frame_stream: &mut FramedWrite<
        tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
        MqttCodec,
    >,
) -> bool {
    if connection_manager.tcp_connect_num_check() {
        let packet_wrapper = MqttPacketWrapper {
            protocol_version: MqttProtocol::Mqtt5.into(),
            packet: response_packet_mqtt_distinct_by_reason(
                &MqttProtocol::Mqtt5,
                Some(DisconnectReasonCode::QuotaExceeded),
            ),
        };
        match write_frame_stream.send(packet_wrapper).await {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        }

        match write_frame_stream.close().await {
            Ok(_) => {
                error!(
                    "tcp connection failed to establish from IP: {}",
                    addr.to_string()
                );
            }
            Err(e) => error!("{}", e),
        }
        return false;
    }

    if is_connection_rate_exceeded() {
        let packet_wrapper = MqttPacketWrapper {
            protocol_version: MqttProtocol::Mqtt5.into(),
            packet: response_packet_mqtt_distinct_by_reason(
                &MqttProtocol::Mqtt5,
                Some(DisconnectReasonCode::ConnectionRateExceeded),
            ),
        };
        match write_frame_stream.send(packet_wrapper).await {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        }

        match write_frame_stream.close().await {
            Ok(_) => {
                error!(
                    "tcp connection failed to establish from IP: {}",
                    addr.to_string()
                );
            }
            Err(e) => error!("{}", e),
        }
        return false;
    }
    true
}

#[allow(clippy::too_many_arguments)]
pub fn connect_validator(
    protocol: &MqttProtocol,
    cluster: &MqttClusterDynamicConfig,
    connect: &Connect,
    connect_properties: &Option<ConnectProperties>,
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
    login: &Option<Login>,
    addr: &SocketAddr,
) -> Option<MqttPacket> {
    if cluster.security.is_self_protection_status {
        return Some(response_packet_mqtt_connect_fail(
            protocol,
            ConnectReturnCode::ServerBusy,
            connect_properties,
            Some(MqttBrokerError::ClusterIsInSelfProtection.to_string()),
        ));
    }

    if is_ip_blacklist(addr) {
        return Some(response_packet_mqtt_connect_fail(
            protocol,
            ConnectReturnCode::Banned,
            connect_properties,
            None,
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

    let cluster = cache_manager.get_cluster_info();
    let max_packet_size =
        min(cluster.protocol.max_packet_size, connection.max_packet_size) as usize;
    if publish.payload.len() > max_packet_size {
        if is_puback {
            return Some(response_packet_mqtt_puback_fail(
                protocol,
                connection,
                publish.pkid,
                PubAckReason::PayloadFormatInvalid,
                Some(MqttBrokerError::PacketLengthError(publish.payload.len()).to_string()),
            ));
        } else {
            return Some(response_packet_mqtt_pubrec_fail(
                protocol,
                connection,
                publish.pkid,
                PubRecReason::PayloadFormatInvalid,
                Some(MqttBrokerError::PacketLengthError(publish.payload.len()).to_string()),
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
                        Some(MqttBrokerError::PacketLengthError(publish.payload.len()).to_string()),
                    ));
                } else {
                    return Some(response_packet_mqtt_pubrec_fail(
                        protocol,
                        connection,
                        publish.pkid,
                        PubRecReason::PayloadFormatInvalid,
                        Some(MqttBrokerError::PacketLengthError(publish.payload.len()).to_string()),
                    ));
                }
            }
        }
    }
    if authentication_acl() {
        if is_puback {
            return Some(response_packet_mqtt_puback_fail(
                protocol,
                connection,
                publish.pkid,
                PubAckReason::NotAuthorized,
                None,
            ));
        } else {
            return Some(response_packet_mqtt_pubrec_fail(
                protocol,
                connection,
                publish.pkid,
                PubRecReason::NotAuthorized,
                None,
            ));
        }
    }

    if is_flow_control(protocol, publish.qos)
        && connection.get_recv_qos_message() >= cluster.protocol.receive_max as isize
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

    if let Some(properties) = publish_properties {
        if let Some(alias) = properties.topic_alias {
            let cluster = cache_manager.get_cluster_info();
            if alias > cluster.protocol.topic_alias_max {
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
    _cache_manager: &Arc<CacheManager>,
    _client_pool: &Arc<ClientPool>,
    connection: &MQTTConnection,
    subscribe: &Subscribe,
) -> Option<MqttPacket> {
    // match pkid_exists(
    //     cache_manager,
    //     client_pool,
    //     &connection.client_id,
    //     subscribe.packet_identifier,
    // )
    // .await
    // {
    //     Ok(res) => {
    //         if res {
    //             return Some(response_packet_mqtt_suback(
    //                 protocol,
    //                 connection,
    //                 subscribe.packet_identifier,
    //                 vec![SubscribeReasonCode::PkidInUse],
    //                 None,
    //             ));
    //         }
    //     }
    //     Err(e) => {
    //         return Some(response_packet_mqtt_suback(
    //             protocol,
    //             connection,
    //             subscribe.packet_identifier,
    //             vec![SubscribeReasonCode::Unspecified],
    //             Some(e.to_string()),
    //         ));
    //     }
    // };

    let mut return_codes: Vec<SubscribeReasonCode> = Vec::new();
    for filter in subscribe.filters.clone() {
        if !sub_path_validator(filter.path) {
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

    if authentication_acl() {
        return Some(response_packet_mqtt_suback(
            protocol,
            connection,
            subscribe.packet_identifier,
            vec![SubscribeReasonCode::NotAuthorized],
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

    None
}

pub async fn un_subscribe_validator(
    client_id: &str,
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    connection: &MQTTConnection,
    un_subscribe: &Unsubscribe,
) -> Option<MqttPacket> {
    match pkid_exists(
        cache_manager,
        client_pool,
        &connection.client_id,
        un_subscribe.pkid,
    )
    .await
    {
        Ok(res) => {
            if res {
                return Some(response_packet_mqtt_unsuback(
                    connection,
                    un_subscribe.pkid,
                    vec![UnsubAckReason::PacketIdentifierInUse],
                    None,
                ));
            }
        }
        Err(e) => {
            return Some(response_packet_mqtt_unsuback(
                connection,
                un_subscribe.pkid,
                vec![UnsubAckReason::UnspecifiedError],
                Some(e.to_string()),
            ));
        }
    };

    let mut return_codes: Vec<UnsubAckReason> = Vec::new();
    for filter in un_subscribe.filters.clone() {
        if !sub_path_validator(filter) {
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

    if authentication_acl() {
        return Some(response_packet_mqtt_unsuback(
            connection,
            un_subscribe.pkid,
            vec![UnsubAckReason::NotAuthorized],
            None,
        ));
    }

    for path in un_subscribe.filters.clone() {
        if let Some(sub_list) = cache_manager.subscribe_filter.get_mut(client_id) {
            if !sub_list.contains_key(&path) {
                return Some(response_packet_mqtt_unsuback(
                    connection,
                    un_subscribe.pkid,
                    vec![UnsubAckReason::NoSubscriptionExisted],
                    Some(MqttBrokerError::SubscriptionPathNotExists(path).to_string()),
                ));
            }
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
    cluster: &MqttClusterDynamicConfig,
) -> u32 {
    if let Some(properties) = connect_properties {
        if let Some(size) = properties.max_packet_size {
            return min(size, cluster.protocol.max_packet_size);
        }
    }
    cluster.protocol.max_packet_size
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
