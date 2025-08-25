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

use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::CacheManager;
use crate::handler::command::{create_command, CommandContext};
use crate::handler::error::MqttBrokerError;
use crate::security::AuthDriver;
use crate::subscribe::manager::SubscribeManager;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use axum_extra::headers::UserAgent;
use axum_extra::TypedHeader;
use axum_server::tls_rustls::RustlsConfig;
use bytes::{BufMut, BytesMut};
use common_base::tools::now_mills;
use common_config::broker::broker_config;
use delay_message::DelayMessageManager;
use futures_util::stream::StreamExt;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use network_server::command::ArcCommandAdapter;
use network_server::common::connection_manager::ConnectionManager;
use observability::mqtt::server::record_ws_request_duration;
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::MqttPacket;
use protocol::robust::{RobustMQPacket, RobustMQPacketWrapper};
use schema_register::schema::SchemaRegisterManager;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self};
use tracing::{error, info, warn};

pub const ROUTE_ROOT: &str = "/mqtt";

#[derive(Clone)]
pub struct WebSocketServerContext {
    pub connection_manager: Arc<ConnectionManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub cache_manager: Arc<CacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub stop_sx: broadcast::Sender<bool>,
    pub message_storage_adapter: ArcStorageAdapter,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub schema_manager: Arc<SchemaRegisterManager>,
    pub auth_driver: Arc<AuthDriver>,
}

#[derive(Clone)]
pub struct WebSocketServerState {
    subscribe_manager: Arc<SubscribeManager>,
    cache_manager: Arc<CacheManager>,
    message_storage_adapter: ArcStorageAdapter,
    delay_message_manager: Arc<DelayMessageManager>,
    client_pool: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
    schema_manager: Arc<SchemaRegisterManager>,
    auth_driver: Arc<AuthDriver>,
}

impl WebSocketServerState {
    pub fn new(context: WebSocketServerContext) -> Self {
        Self {
            connection_manager: context.connection_manager,
            subscribe_manager: context.subscribe_manager,
            cache_manager: context.cache_manager,
            message_storage_adapter: context.message_storage_adapter,
            delay_message_manager: context.delay_message_manager,
            schema_manager: context.schema_manager,
            client_pool: context.client_pool,
            stop_sx: context.stop_sx,
            auth_driver: context.auth_driver,
        }
    }
}

pub async fn websocket_server(state: WebSocketServerState) {
    let config = broker_config();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.mqtt_server.websocket_port)
        .parse()
        .unwrap();
    let app = routes_v1(state);
    info!(
        "Broker WebSocket Server start success. port:{}",
        config.mqtt_server.websocket_port
    );
    match axum_server::bind(ip)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
    {
        Ok(()) => {}
        Err(e) => panic!("{}", e.to_string()),
    }
}

pub async fn websockets_server(state: WebSocketServerState) {
    let config = broker_config();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.mqtt_server.websockets_port)
        .parse()
        .unwrap();
    let app = routes_v1(state);

    let tls_config = match RustlsConfig::from_pem_file(
        PathBuf::from(config.runtime.tls_cert.clone()),
        PathBuf::from(config.runtime.tls_key.clone()),
    )
    .await
    {
        Ok(cf) => cf,
        Err(e) => {
            panic!("{}", e.to_string());
        }
    };

    info!(
        "Broker WebSocket TLS Server start success. port:{}",
        config.mqtt_server.websockets_port
    );
    if let Err(e) = axum_server::bind_rustls(ip, tls_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
    {
        panic!("{}", e.to_string());
    }
}

fn routes_v1(state: WebSocketServerState) -> Router {
    let mqtt_ws = Router::new().route(ROUTE_ROOT, get(ws_handler));

    let app = Router::new().merge(mqtt_ws);
    app.with_state(state)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketServerState>,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown Source")
    };

    info!("websocket `{user_agent}` at {addr} connected.");
    let context = CommandContext {
        cache_manager: state.cache_manager.clone(),
        message_storage_adapter: state.message_storage_adapter.clone(),
        delay_message_manager: state.delay_message_manager.clone(),
        subscribe_manager: state.subscribe_manager.clone(),
        client_pool: state.client_pool.clone(),
        connection_manager: state.connection_manager.clone(),
        schema_manager: state.schema_manager.clone(),
        auth_driver: state.auth_driver.clone(),
    };

    let command = create_command(context);
    let codec = MqttCodec::new(None);
    ws.protocols(["mqtt", "mqttv3.1"])
        .on_upgrade(move |socket| {
            handle_socket(
                socket,
                addr,
                command,
                codec,
                state.connection_manager.clone(),
                state.stop_sx.clone(),
            )
        })
}

async fn handle_socket(
    socket: WebSocket,
    addr: SocketAddr,
    command: ArcCommandAdapter,
    mut codec: MqttCodec,
    connection_manager: Arc<ConnectionManager>,
    stop_sx: broadcast::Sender<bool>,
) {
    let (sender, mut receiver) = socket.split();
    let tcp_connection = NetworkConnection::new(NetworkConnectionType::WebSocket, addr, None);
    connection_manager.add_mqtt_websocket_write(tcp_connection.connection_id, sender);
    connection_manager.add_connection(tcp_connection.clone());
    let mut stop_rx = stop_sx.subscribe();

    loop {
        select! {
            val = stop_rx.recv() =>{
                if let Ok(flag) = val {
                    if flag {
                        break;
                    }
                }
            },
            val = receiver.next()=>{
                if let Some(msg) = val{
                    match msg {
                        Ok(Message::Binary(data)) => {
                            if let Err(e) = process_socket_packet_by_binary(tcp_connection.connection_id(), &connection_manager,&mut codec, command.clone(), addr,data).await{
                                error!("Websocket failed to parse MQTT protocol packet with error message :{e:?}");
                            }
                        }
                        Ok(Message::Text(data)) => {
                            warn!(
                                "websocket server receives a TEXT message with the following content: {data}"
                            );
                        }
                        Ok(Message::Ping(data)) => {
                            info!(
                                "websocket server receives a Ping message with the following content: {data:?}"
                            );
                        }
                        Ok(Message::Pong(data)) => {
                            info!(
                                "websocket server receives a Pong message with the following content: {data:?}"
                            );
                        }
                        Ok(Message::Close(data)) => {
                            if let Some(cf) = data {
                                info!(
                                    ">>> {} sent close with code {} and reason `{}`",
                                    addr, cf.code, cf.reason
                                );
                            } else {
                                info!(
                                    ">>> {addr} somehow sent close message without CloseFrame"
                                );
                            }
                            break;
                        }
                        Err(e) => {
                            warn!("websocket server parsing request packet error, error message :{e:?}");
                        },
                    }
                }
            }
        }
    }
}

async fn process_socket_packet_by_binary(
    connection_id: u64,
    connection_manager: &Arc<ConnectionManager>,
    codec: &mut MqttCodec,
    command: ArcCommandAdapter,
    addr: SocketAddr,
    data: Vec<u8>,
) -> ResultMqttBrokerError {
    let receive_ms = now_mills();
    let mut buf = BytesMut::with_capacity(data.len());
    buf.put(data.as_slice());
    if let Some(packet) = codec.decode_data(&mut buf)? {
        info!("recv websocket packet:{packet:?}");
        let tcp_connection = if let Some(conn) = connection_manager.get_connect(connection_id) {
            conn
        } else {
            return Err(MqttBrokerError::NotFoundConnectionInCache(connection_id));
        };
        if let Some(resp_pkg) = command
            .apply(
                tcp_connection.clone(),
                addr,
                RobustMQPacket::MQTT(packet.clone()),
            )
            .await
        {
            if let MqttPacket::Connect(protocol, _, _, _, _, _) = packet {
                connection_manager
                    .set_mqtt_connect_protocol(tcp_connection.connection_id, protocol);
            }

            let mut response_buff = BytesMut::new();
            let packet_wrapper = MqttPacketWrapper {
                protocol_version: tcp_connection.get_protocol().into(),
                packet: resp_pkg.packet.get_mqtt_packet().unwrap(),
            };
            codec.encode_data(packet_wrapper.clone(), &mut response_buff)?;
            let response_ms = now_mills();

            if let Err(e) = connection_manager
                .write_websocket_frame(
                    tcp_connection.connection_id,
                    RobustMQPacketWrapper::from_mqtt(packet_wrapper),
                    Message::Binary(response_buff.to_vec()),
                )
                .await
            {
                error!("websocket returns failure to write the packet to the client with error message {e:?}");
                connection_manager
                    .close_connect(tcp_connection.connection_id)
                    .await;
            }
            record_ws_request_duration(receive_ms, response_ms);
        } else {
            info!("{}", "No backpacking is required for this request");
        }
    }

    Ok(())
}
