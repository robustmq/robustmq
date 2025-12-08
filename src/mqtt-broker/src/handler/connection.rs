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
use crate::handler::flow_control::is_connection_rate_exceeded;
use crate::handler::response::response_packet_mqtt_distinct_by_reason;
use crate::handler::tool::ResultMqttBrokerError;
use crate::storage::session::SessionStorage;
use crate::subscribe::manager::SubscribeManager;
use common_base::tools::{now_second, unique_id};
use futures_util::SinkExt;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::connection::{ConnectionConfig, MQTTConnection};
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::{Connect, ConnectProperties, DisconnectReasonCode, MqttProtocol};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncWrite, AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio_util::codec::FramedWrite;
use tracing::{error, warn};

pub const REQUEST_RESPONSE_PREFIX_NAME: &str = "/sys/request_response/";
pub const DISCONNECT_FLAG_NOT_DELETE_SESSION: &str = "DISCONNECT_FLAG_NOT_DELETE_SESSION";

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

pub fn get_client_id(client_id: &str) -> (String, bool) {
    if client_id.is_empty() {
        (unique_id(), true)
    } else {
        (client_id.to_owned(), false)
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

pub fn is_delete_session(user_properties: &Vec<(String, String)>) -> bool {
    for (key, value) in user_properties {
        if *key == *DISCONNECT_FLAG_NOT_DELETE_SESSION && value == "true" {
            return false;
        }
    }
    true
}

pub async fn disconnect_connection(
    client_id: &str,
    connect_id: u64,
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    connection_manager: &Arc<ConnectionManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    delete_session: bool,
) -> ResultMqttBrokerError {
    let session_storage = SessionStorage::new(client_pool.clone());
    if delete_session {
        session_storage.delete_session(client_id.to_owned()).await?;
        cache_manager.remove_session(client_id);
        subscribe_manager.remove_by_client_id(client_id);
    } else {
        cache_manager.update_session_connect_id(client_id, None);
        session_storage
            .update_session(client_id.to_owned(), 0, 0, 0, now_second())
            .await?;
    }

    connection_manager.close_connect(connect_id).await;
    cache_manager.remove_connection(connect_id);
    Ok(())
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

    use crate::handler::tool::test_build_mqtt_cache_manager;

    use super::{
        build_connection, get_client_id, response_information, MQTTConnection,
        REQUEST_RESPONSE_PREFIX_NAME,
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
        let client_id = "".to_string();
        let (new_client_id, is_new) = get_client_id(&client_id);
        assert!(is_new);
        assert!(!new_client_id.is_empty());

        let client_id = "client_id-***".to_string();
        let (new_client_id, is_new) = get_client_id(&client_id);
        assert!(!is_new);
        assert_eq!(new_client_id, client_id);
        assert!(!new_client_id.is_empty());
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
