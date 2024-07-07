use super::{
    cache_manager::CacheManager,
    connection::Connection,
    error::MQTTBrokerError,
    flow_control::{is_connection_rate_exceeded, is_self_protection_status},
    pkid::pkid_exists,
    response_packet::{
        response_packet_matt5_connect_fail, response_packet_matt5_puback_fail,
        response_packet_matt5_pubrec_fail, response_packet_matt_distinct,
    },
    topic::topic_name_validator,
};
use crate::{
    core::flow_control::is_publish_rate_exceeded,
    security::authentication::{authentication_acl, is_ip_blacklist},
    server::tcp::connection_manager::ConnectionManager,
};
use clients::poll::ClientPool;
use common_base::{errors::RobustMQError, log::error};
use futures::SinkExt;
use metadata_struct::mqtt::cluster::MQTTCluster;
use protocol::mqtt::{
    codec::{MQTTPacketWrapper, MqttCodec},
    common::{
        Connect, ConnectProperties, ConnectReturnCode, DisconnectReasonCode, LastWill,
        LastWillProperties, Login, MQTTPacket, MQTTProtocol, PubAckReason, PubRecReason, Publish,
        PublishProperties, QoS,
    },
};
use std::{cmp::min, net::SocketAddr, sync::Arc};
use tokio_util::codec::FramedWrite;

pub async fn establish_connection_check(
    addr: &SocketAddr,
    connection_manager: &Arc<ConnectionManager>,
    write_frame_stream: &mut FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, MqttCodec>,
) -> bool {
    if connection_manager.connect_num_check() {
        let packet_wrapper = MQTTPacketWrapper {
            protocol_version: MQTTProtocol::MQTT5.into(),
            packet: response_packet_matt_distinct(DisconnectReasonCode::QuotaExceeded, None),
        };
        match write_frame_stream.send(packet_wrapper).await {
            Ok(_) => {}
            Err(e) => error(e.to_string()),
        }

        match write_frame_stream.close().await {
            Ok(_) => {
                error(format!(
                    "tcp connection failed to establish from IP: {}",
                    addr.to_string()
                ));
            }
            Err(e) => error(e.to_string()),
        }
        return false;
    }

    if is_connection_rate_exceeded() {
        let packet_wrapper = MQTTPacketWrapper {
            protocol_version: MQTTProtocol::MQTT4.into(),
            packet: response_packet_matt_distinct(
                DisconnectReasonCode::ConnectionRateExceeded,
                None,
            ),
        };
        match write_frame_stream.send(packet_wrapper).await {
            Ok(_) => {}
            Err(e) => error(e.to_string()),
        }

        match write_frame_stream.close().await {
            Ok(_) => {
                error(format!(
                    "tcp connection failed to establish from IP: {}",
                    addr.to_string()
                ));
            }
            Err(e) => error(e.to_string()),
        }
        return false;
    }
    return true;
}

pub fn connect_validator(
    cluster: &MQTTCluster,
    connect: &Connect,
    connect_properties: &Option<ConnectProperties>,
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
    login: &Option<Login>,
    addr: &SocketAddr,
) -> Option<MQTTPacket> {
    if is_self_protection_status() {
        return Some(response_packet_matt5_connect_fail(
            ConnectReturnCode::ServerBusy,
            &connect_properties,
            None,
        ));
    }

    if is_ip_blacklist(addr) {
        return Some(response_packet_matt5_connect_fail(
            ConnectReturnCode::Banned,
            &connect_properties,
            None,
        ));
    }

    if !connect.client_id.is_empty() && !client_id_validator(&connect.client_id) {
        return Some(response_packet_matt5_connect_fail(
            ConnectReturnCode::ClientIdentifierNotValid,
            connect_properties,
            None,
        ));
    }

    if let Some(login_info) = login {
        if !username_validator(&login_info.username) || !password_validator(&login_info.password) {
            return Some(response_packet_matt5_connect_fail(
                ConnectReturnCode::BadUserNamePassword,
                connect_properties,
                None,
            ));
        }
    }

    if let Some(will) = last_will {
        if will.topic.is_empty() {
            return Some(response_packet_matt5_connect_fail(
                ConnectReturnCode::TopicNameInvalid,
                connect_properties,
                None,
            ));
        }

        let topic_name = match String::from_utf8(will.topic.to_vec()) {
            Ok(da) => da,
            Err(e) => {
                return Some(response_packet_matt5_connect_fail(
                    ConnectReturnCode::TopicNameInvalid,
                    connect_properties,
                    Some(e.to_string()),
                ));
            }
        };

        match topic_name_validator(&topic_name) {
            Ok(()) => {}
            Err(e) => {
                Some(response_packet_matt5_connect_fail(
                    ConnectReturnCode::TopicNameInvalid,
                    connect_properties,
                    Some(e.to_string()),
                ));
            }
        }

        if will.message.is_empty() {
            return Some(response_packet_matt5_connect_fail(
                ConnectReturnCode::PayloadFormatInvalid,
                connect_properties,
                None,
            ));
        }

        let max_packet_size = connection_max_packet_size(connect_properties, cluster) as usize;
        if will.message.len() > max_packet_size {
            return Some(response_packet_matt5_connect_fail(
                ConnectReturnCode::PacketTooLarge,
                connect_properties,
                None,
            ));
        }

        if let Some(will_properties) = last_will_properties {
            if let Some(payload_format) = will_properties.payload_format_indicator {
                if payload_format == 1 {
                    if !std::str::from_utf8(&will.message.to_vec().as_slice()).is_ok() {
                        return Some(response_packet_matt5_connect_fail(
                            ConnectReturnCode::PayloadFormatInvalid,
                            connect_properties,
                            None,
                        ));
                    };
                }
            }
        }
    }
    return None;
}

pub async fn publish_validator(
    cache_manager: &Arc<CacheManager>,
    client_poll: &Arc<ClientPool>,
    connection: &Connection,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> Option<MQTTPacket> {
    let is_puback = publish.qos != QoS::ExactlyOnce;
    if publish.topic.is_empty() {
        if is_puback {
            return Some(response_packet_matt5_puback_fail(
                connection,
                publish.pkid,
                PubAckReason::TopicNameInvalid,
                None,
            ));
        } else {
            return Some(response_packet_matt5_pubrec_fail(
                connection,
                publish.pkid,
                PubRecReason::TopicNameInvalid,
                None,
            ));
        }
    }

    let topic_name = match String::from_utf8(publish.topic.to_vec()) {
        Ok(da) => da,
        Err(e) => {
            if is_puback {
                return Some(response_packet_matt5_puback_fail(
                    connection,
                    publish.pkid,
                    PubAckReason::TopicNameInvalid,
                    Some(e.to_string()),
                ));
            } else {
                return Some(response_packet_matt5_pubrec_fail(
                    connection,
                    publish.pkid,
                    PubRecReason::TopicNameInvalid,
                    Some(e.to_string()),
                ));
            }
        }
    };

    match topic_name_validator(&topic_name) {
        Ok(()) => {}
        Err(e) => {
            if is_puback {
                return Some(response_packet_matt5_puback_fail(
                    connection,
                    publish.pkid,
                    PubAckReason::TopicNameInvalid,
                    Some(e.to_string()),
                ));
            } else {
                return Some(response_packet_matt5_pubrec_fail(
                    connection,
                    publish.pkid,
                    PubRecReason::TopicNameInvalid,
                    Some(e.to_string()),
                ));
            }
        }
    }

    if publish.payload.is_empty() {
        if is_puback {
            return Some(response_packet_matt5_puback_fail(
                connection,
                publish.pkid,
                PubAckReason::PayloadFormatInvalid,
                None,
            ));
        } else {
            return Some(response_packet_matt5_pubrec_fail(
                connection,
                publish.pkid,
                PubRecReason::PayloadFormatInvalid,
                None,
            ));
        }
    }

    if publish.qos == QoS::ExactlyOnce {
        match pkid_exists(
            cache_manager,
            client_poll,
            &connection.client_id,
            publish.pkid,
        )
        .await
        {
            Ok(res) => {
                if res {
                    return Some(response_packet_matt5_pubrec_fail(
                        connection,
                        publish.pkid,
                        PubRecReason::PayloadFormatInvalid,
                        None,
                    ));
                }
            }
            Err(e) => {
                return Some(response_packet_matt5_pubrec_fail(
                    connection,
                    publish.pkid,
                    PubRecReason::UnspecifiedError,
                    Some(e.to_string()),
                ));
            }
        };
    }

    let cluster = cache_manager.get_cluster_info();
    let max_packet_size = min(cluster.max_packet_size, connection.max_packet_size) as usize;
    if publish.payload.len() > max_packet_size {
        if is_puback {
            return Some(response_packet_matt5_puback_fail(
                connection,
                publish.pkid,
                PubAckReason::PayloadFormatInvalid,
                Some(RobustMQError::PacketLenthError(publish.payload.len()).to_string()),
            ));
        } else {
            return Some(response_packet_matt5_pubrec_fail(
                connection,
                publish.pkid,
                PubRecReason::PayloadFormatInvalid,
                Some(RobustMQError::PacketLenthError(publish.payload.len()).to_string()),
            ));
        }
    }

    if let Some(properties) = publish_properties {
        if let Some(payload_format) = properties.payload_format_indicator {
            if payload_format == 1 {
                if !std::str::from_utf8(&publish.payload.to_vec().as_slice()).is_ok() {
                    if is_puback {
                        return Some(response_packet_matt5_puback_fail(
                            connection,
                            publish.pkid,
                            PubAckReason::PayloadFormatInvalid,
                            Some(
                                RobustMQError::PacketLenthError(publish.payload.len()).to_string(),
                            ),
                        ));
                    } else {
                        return Some(response_packet_matt5_pubrec_fail(
                            connection,
                            publish.pkid,
                            PubRecReason::PayloadFormatInvalid,
                            Some(
                                RobustMQError::PacketLenthError(publish.payload.len()).to_string(),
                            ),
                        ));
                    }
                }
            }
        }
    }
    if authentication_acl() {
        if is_puback {
            return Some(response_packet_matt5_puback_fail(
                connection,
                publish.pkid,
                PubAckReason::NotAuthorized,
                None,
            ));
        } else {
            return Some(response_packet_matt5_pubrec_fail(
                connection,
                publish.pkid,
                PubRecReason::NotAuthorized,
                None,
            ));
        }
    }

    if is_publish_rate_exceeded() {
        if is_puback {
            return Some(response_packet_matt5_puback_fail(
                connection,
                publish.pkid,
                PubAckReason::QuotaExceeded,
                None,
            ));
        } else {
            return Some(response_packet_matt5_pubrec_fail(
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
            if alias > cluster.topic_alias_max {
                if is_puback {
                    return Some(response_packet_matt5_puback_fail(
                        connection,
                        publish.pkid,
                        PubAckReason::UnspecifiedError,
                        Some(MQTTBrokerError::TopicAliasTooLong(alias).to_string()),
                    ));
                } else {
                    return Some(response_packet_matt5_pubrec_fail(
                        connection,
                        publish.pkid,
                        PubRecReason::UnspecifiedError,
                        Some(MQTTBrokerError::TopicAliasTooLong(alias).to_string()),
                    ));
                }
            }
        }
    }

    return None;
}

pub fn subscribe_validator() -> Option<MQTTPacket> {
    return None;
}

pub fn is_request_problem_info(connect_properties: &Option<ConnectProperties>) -> bool {
    if let Some(properties) = connect_properties {
        if let Some(problem_info) = properties.request_problem_info {
            return problem_info == 1;
        }
    }
    return false;
}

pub fn connection_max_packet_size(
    connect_properties: &Option<ConnectProperties>,
    cluster: &MQTTCluster,
) -> u32 {
    if let Some(properties) = connect_properties {
        if let Some(size) = properties.max_packet_size {
            return min(size, cluster.max_packet_size());
        }
    }
    return cluster.max_packet_size();
}

pub fn client_id_validator(client_id: &String) -> bool {
    if client_id.len() < 5 {
        return false;
    }
    return true;
}

pub fn username_validator(username: &String) -> bool {
    if username.is_empty() {
        return false;
    }
    return true;
}

pub fn password_validator(password: &String) -> bool {
    if password.is_empty() {
        return false;
    }
    return true;
}
