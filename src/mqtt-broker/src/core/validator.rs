use super::{
    response_packet::{response_packet_matt5_connect_fail, response_packet_matt_distinct},
    topic::topic_name_validator,
};
use crate::server::tcp::connection_manager::ConnectionManager;
use common_base::log::error;
use futures::SinkExt;
use metadata_struct::mqtt::cluster::MQTTCluster;
use protocol::mqtt::{
    codec::{MQTTPacketWrapper, MqttCodec},
    common::{
        Connect, ConnectProperties, ConnectReturnCode, DisconnectReasonCode, LastWill,
        LastWillProperties, Login, MQTTPacket, MQTTProtocol,
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
            protocol_version: MQTTProtocol::MQTT4.into(),
            packet: response_packet_matt_distinct(DisconnectReasonCode::QuotaExceeded),
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

    if connection_manager.connect_rate_check() {
        let packet_wrapper = MQTTPacketWrapper {
            protocol_version: MQTTProtocol::MQTT4.into(),
            packet: response_packet_matt_distinct(DisconnectReasonCode::ConnectionRateExceeded),
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

pub fn connect_params_validator(
    cluster: &MQTTCluster,
    connect: &Connect,
    connect_properties: &Option<ConnectProperties>,
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
    login: &Option<Login>,
) -> Option<MQTTPacket> {
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

pub fn publish_params_validator() -> Option<String> {
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
