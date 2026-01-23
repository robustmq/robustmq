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

use super::cache::MQTTCacheManager;
use super::keep_alive::client_keep_live_time;
use crate::core::error::MqttBrokerError;
use crate::core::flow_control::is_connection_rate_exceeded;
use crate::core::response::{
    response_packet_mqtt_connect_fail, response_packet_mqtt_distinct_by_reason,
};
use crate::core::session::delete_session_by_local;
use crate::core::tool::ResultMqttBrokerError;
use crate::storage::session::SessionStorage;
use crate::subscribe::manager::SubscribeManager;
use common_base::tools::{now_second, unique_id};
use futures_util::SinkExt;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::connection::{ConnectionConfig, MQTTConnection};
use metadata_struct::mqtt::session::MqttSession;
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::{
    Connect, ConnectProperties, ConnectReturnCode, DisconnectProperties, DisconnectReasonCode,
    MqttPacket, MqttProtocol,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncWrite, AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio_util::codec::FramedWrite;
use tracing::{error, warn};

pub const REQUEST_RESPONSE_PREFIX_NAME: &str = "/sys/request_response/";

#[derive(Clone)]
pub struct DisconnectConnectionContext {
    pub cache_manager: Arc<MQTTCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub connection_manager: Arc<ConnectionManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub disconnect_properties: Option<DisconnectProperties>,
    pub connection: MQTTConnection,
    pub session: MqttSession,
    pub protocol: MqttProtocol,
}

pub async fn build_connection(
    connect_id: u64,
    client_id: String,
    cache_manager: &Arc<MQTTCacheManager>,
    connect: &Connect,
    connect_properties: &Option<ConnectProperties>,
    addr: &SocketAddr,
) -> MQTTConnection {
    let config = cache_manager.broker_cache.get_cluster_config().await;
    let keep_alive = client_keep_live_time(cache_manager, connect.keep_alive).await;
    let (client_receive_maximum, max_packet_size, topic_alias_max, request_problem_info) =
        if let Some(properties) = connect_properties {
            let client_receive_maximum = if let Some(value) = properties.receive_maximum {
                value
            } else {
                config.mqtt_protocol_config.receive_max
            };

            let max_packet_size = if let Some(value) = properties.max_packet_size {
                std::cmp::min(value, config.mqtt_protocol_config.max_packet_size)
            } else {
                config.mqtt_protocol_config.max_packet_size
            };

            let topic_alias_max = if let Some(value) = properties.topic_alias_max {
                std::cmp::min(value, config.mqtt_protocol_config.topic_alias_max)
            } else {
                config.mqtt_protocol_config.topic_alias_max
            };

            let request_problem_info = properties.request_problem_info.unwrap_or_default();

            (
                client_receive_maximum,
                max_packet_size,
                topic_alias_max,
                request_problem_info,
            )
        } else {
            (
                config.mqtt_protocol_config.receive_max,
                config.mqtt_protocol_config.max_packet_size,
                config.mqtt_protocol_config.topic_alias_max,
                0,
            )
        };

    let config = ConnectionConfig {
        connect_id,
        client_id: client_id.clone(),
        receive_maximum: client_receive_maximum,
        max_packet_size,
        topic_alias_max,
        request_problem_info,
        keep_alive,
        source_ip_addr: addr.to_string(),
    };
    MQTTConnection::new(config)
}

pub fn get_client_id(
    protocol: &MqttProtocol,
    clean_session: bool,
    client_id: &str,
) -> (Option<(String, bool)>, Option<MqttPacket>) {
    match protocol {
        MqttProtocol::Mqtt3 => {
            if client_id.is_empty() {
                return (
                    None,
                    Some(response_packet_mqtt_connect_fail(
                        protocol,
                        ConnectReturnCode::IdentifierRejected,
                        &None,
                        None,
                    )),
                );
            }
            // For MQTT 3.x, client_id is provided by client (non-empty here),
            // so it is NOT an auto-assigned client identifier.
            (Some((client_id.to_owned(), false)), None)
        }

        MqttProtocol::Mqtt4 => {
            if client_id.is_empty() && !clean_session {
                return (
                    None,
                    Some(response_packet_mqtt_connect_fail(
                        protocol,
                        ConnectReturnCode::IdentifierRejected,
                        &None,
                        None,
                    )),
                );
            }

            if client_id.is_empty() {
                (Some((unique_id(), true)), None)
            } else {
                (Some((client_id.to_owned(), false)), None)
            }
        }

        MqttProtocol::Mqtt5 => {
            if client_id.is_empty() {
                (Some((unique_id(), true)), None)
            } else {
                (Some((client_id.to_owned(), false)), None)
            }
        }
    }
}

pub fn response_information(connect_properties: &Option<ConnectProperties>) -> Option<String> {
    if let Some(properties) = connect_properties {
        if let Some(request_response_info) = properties.request_response_info {
            if request_response_info == 1 {
                return Some(REQUEST_RESPONSE_PREFIX_NAME.to_string());
            }
        }
    }
    None
}

/// Handle connection teardown and persist session state according to MQTT semantics.
///
/// - MQTT 3.1 / 3.1.1: if `Clean Session = 1`, the session is deleted on disconnect; otherwise the
///   session is kept and marked offline.
/// - MQTT 5.0: the effective Session Expiry Interval used at disconnect time is:
///   `DISCONNECT.properties.session_expiry_interval` (if present) otherwise the value stored in the
///   session (set during CONNECT). If the effective expiry is 0, delete the session immediately;
///   otherwise keep the session, mark it offline, and persist the effective expiry.
pub async fn disconnect_connection(context: DisconnectConnectionContext) -> ResultMqttBrokerError {
    let session_storage = SessionStorage::new(context.client_pool.clone());
    let session_expiry_interval =
        get_session_expiry_interval(&context.session, &context.disconnect_properties);
    if is_delete_session(
        &context.connection,
        &context.protocol,
        session_expiry_interval,
    ) {
        session_storage
            .delete_session(context.connection.client_id.clone())
            .await?;
        delete_session_by_local(
            &context.cache_manager,
            &context.subscribe_manager,
            &context.connection.client_id,
        );
    } else {
        let mut new_session = context.session.clone();
        if context.protocol.is_mqtt5() {
            // MQTT5: DISCONNECT Session Expiry Interval (if present) overrides the session expiry.
            new_session.session_expiry_interval = session_expiry_interval as u64;
        }
        new_session.connection_id = None;
        new_session.broker_id = None;
        new_session.reconnect_time = None;
        new_session.distinct_time = Some(now_second());
        session_storage
            .set_session(context.connection.client_id.clone(), &new_session)
            .await?;
    }

    context
        .connection_manager
        .close_connect(context.connection.connect_id)
        .await;
    context
        .cache_manager
        .remove_connection(context.connection.connect_id);
    Ok(())
}

pub fn build_server_disconnect_conn_context(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    connection_manager: &Arc<ConnectionManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    connect_id: u64,
    protocol: &MqttProtocol,
) -> Result<DisconnectConnectionContext, MqttBrokerError> {
    let connection = if let Some(connection) = cache_manager.get_connection(connect_id) {
        connection
    } else {
        return Err(MqttBrokerError::NotFoundConnectionInCache(connect_id));
    };

    let session = if let Some(session) = cache_manager.get_session_info(&connection.client_id) {
        session
    } else {
        return Err(MqttBrokerError::SessionDoesNotExist);
    };

    let disconnect_properties = Some(DisconnectProperties {
        session_expiry_interval: Some(session.session_expiry_interval as u32),
        ..Default::default()
    });

    Ok(DisconnectConnectionContext {
        cache_manager: cache_manager.clone(),
        client_pool: client_pool.clone(),
        connection_manager: connection_manager.clone(),
        subscribe_manager: subscribe_manager.clone(),
        disconnect_properties,
        connection,
        session,
        protocol: protocol.clone(),
    })
}

fn is_delete_session(
    connection: &MQTTConnection,
    protocol: &MqttProtocol,
    session_expiry_interval: u32,
) -> bool {
    if (protocol.is_mqtt3() || protocol.is_mqtt4()) && connection.clean_session {
        return true;
    }

    if protocol.is_mqtt5() && session_expiry_interval == 0 {
        return true;
    }
    false
}

fn get_session_expiry_interval(
    session: &MqttSession,
    disconnect_properties: &Option<DisconnectProperties>,
) -> u32 {
    if let Some(properties) = disconnect_properties {
        if let Some(expiry) = properties.session_expiry_interval {
            return expiry;
        }
    }
    session.session_expiry_interval as u32
}

pub async fn tcp_establish_connection_check(
    addr: &SocketAddr,
    connection_manager: &Arc<ConnectionManager>,
    write_frame_stream: &mut FramedWrite<WriteHalf<TcpStream>, MqttCodec>,
) -> bool {
    if let Some(value) =
        handle_tpc_connection_overflow(addr, connection_manager, write_frame_stream).await
    {
        return value;
    }

    if let Some(value) = handle_connection_rate_exceeded(addr, write_frame_stream).await {
        return value;
    }
    true
}

pub async fn tcp_tls_establish_connection_check(
    addr: &SocketAddr,
    connection_manager: &Arc<ConnectionManager>,
    write_frame_stream: &mut FramedWrite<
        WriteHalf<tokio_rustls::server::TlsStream<TcpStream>>,
        MqttCodec,
    >,
) -> bool {
    if let Some(value) =
        handle_tpc_connection_overflow(addr, connection_manager, write_frame_stream).await
    {
        return value;
    }

    if let Some(value) = handle_connection_rate_exceeded(addr, write_frame_stream).await {
        return value;
    }

    true
}

async fn handle_tpc_connection_overflow<T>(
    addr: &SocketAddr,
    connection_manager: &Arc<ConnectionManager>,
    write_frame_stream: &mut FramedWrite<WriteHalf<T>, MqttCodec>,
) -> Option<bool>
where
    T: AsyncWriteExt + AsyncWrite,
{
    if connection_manager.get_tcp_connect_num_check() > 5000 {
        let packet_wrapper = MqttPacketWrapper {
            protocol_version: MqttProtocol::Mqtt5.into(),
            packet: response_packet_mqtt_distinct_by_reason(
                &MqttProtocol::Mqtt5,
                Some(DisconnectReasonCode::QuotaExceeded),
                None,
            ),
        };
        if let Err(e) = write_frame_stream.send(packet_wrapper).await {
            error!("{}", e)
        }
        warn!("Total number of tcp connections at a node exceeds the limit, and the connection is closed. Source IP{:?}",addr);
        return Some(false);
    }
    None
}

async fn handle_connection_rate_exceeded<T>(
    addr: &SocketAddr,
    write_frame_stream: &mut FramedWrite<WriteHalf<T>, MqttCodec>,
) -> Option<bool>
where
    T: AsyncWriteExt + AsyncWrite,
{
    if is_connection_rate_exceeded() {
        let packet_wrapper = MqttPacketWrapper {
            protocol_version: MqttProtocol::Mqtt5.into(),
            packet: response_packet_mqtt_distinct_by_reason(
                &MqttProtocol::Mqtt5,
                Some(DisconnectReasonCode::ConnectionRateExceeded),
                None,
            ),
        };

        if let Err(e) = write_frame_stream.send(packet_wrapper).await {
            error!("{}", e);
        }
        warn!("Total number of tcp connections at a node exceeds the limit, and the connection is closed. Source IP{:?}",addr);
        return Some(false);
    }
    None
}

#[cfg(test)]
mod test {

    use crate::core::tool::test_build_mqtt_cache_manager;

    use super::{
        build_connection, response_information, MQTTConnection, REQUEST_RESPONSE_PREFIX_NAME,
    };
    use common_config::broker::default_broker_config;
    use protocol::mqtt::common::{Connect, ConnectProperties};

    #[tokio::test]
    pub async fn build_connection_test() {
        let connect_id = 1;
        let client_id = "client_id-***".to_string();
        let cluster = default_broker_config();
        println!("{cluster:?}");
        let cache_manager = test_build_mqtt_cache_manager().await;
        let connect = Connect {
            keep_alive: 10,
            client_id: client_id.clone(),
            clean_session: true,
        };
        let connect_properties = ConnectProperties {
            session_expiry_interval: Some(60),
            receive_maximum: Some(100),
            max_packet_size: Some(100),
            request_problem_info: Some(0),
            request_response_info: Some(0),
            topic_alias_max: Some(100),
            user_properties: Vec::new(),
            authentication_method: None,
            authentication_data: None,
        };
        let addr = "0.0.0.0:8080".to_string().parse().unwrap();
        let mut conn = build_connection(
            connect_id,
            client_id.clone(),
            &cache_manager,
            &connect,
            &Some(connect_properties),
            &addr,
        )
        .await;
        assert_eq!(conn.connect_id, connect_id);
        assert_eq!(conn.client_id, client_id);
        assert!(!conn.is_login);
        conn.login_success("".into());
        assert!(conn.is_login);
        assert_eq!(conn.keep_alive, 10);
        assert_eq!(conn.client_max_receive_maximum, 100);
        assert_eq!(conn.max_packet_size, 100);
        assert_eq!(conn.topic_alias_max, 100);
        assert_eq!(conn.request_problem_info, 0);
    }

    #[tokio::test]
    pub async fn get_client_id_test() {
        let (data, resp) =
            super::get_client_id(&protocol::mqtt::common::MqttProtocol::Mqtt3, true, "");
        assert!(data.is_none());
        assert!(resp.is_some());

        let (data, resp) =
            super::get_client_id(&protocol::mqtt::common::MqttProtocol::Mqtt4, false, "");
        assert!(data.is_none());
        assert!(resp.is_some());

        let (data, resp) =
            super::get_client_id(&protocol::mqtt::common::MqttProtocol::Mqtt4, true, "");
        assert!(resp.is_none());
        let (cid, auto) = data.unwrap();
        assert!(!cid.is_empty());
        assert!(auto);

        let (data, resp) =
            super::get_client_id(&protocol::mqtt::common::MqttProtocol::Mqtt5, true, "");
        assert!(resp.is_none());
        let (cid, auto) = data.unwrap();
        assert!(!cid.is_empty());
        assert!(auto);
    }

    #[tokio::test]
    pub async fn response_information_test() {
        let connect_properties = ConnectProperties {
            request_response_info: Some(1),
            ..Default::default()
        };
        let res = response_information(&Some(connect_properties));
        assert_eq!(res.unwrap(), REQUEST_RESPONSE_PREFIX_NAME.to_string());

        let res = response_information(&Some(ConnectProperties::default()));
        assert!(res.is_none());

        let connect_properties = ConnectProperties {
            request_response_info: Some(0),
            ..Default::default()
        };
        let res = response_information(&Some(connect_properties));
        assert!(res.is_none());
    }

    #[tokio::test]
    pub async fn recv_qos_message_num_test() {
        let conn = MQTTConnection::default();
        assert_eq!(conn.get_recv_qos_message(), 0);
        conn.recv_qos_message_incr();
        assert_eq!(conn.get_recv_qos_message(), 1);
        conn.recv_qos_message_decr();
        assert_eq!(conn.get_recv_qos_message(), 0);
    }

    #[tokio::test]
    pub async fn send_qos_message_num_test() {
        let conn = MQTTConnection::default();
        assert_eq!(conn.get_send_qos_message(), 0);
        conn.send_qos_message_incr();
        assert_eq!(conn.get_send_qos_message(), 1);
        conn.send_qos_message_decr();
        assert_eq!(conn.get_send_qos_message(), 0);
    }
}
