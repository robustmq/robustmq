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

use crate::command::ArcCommandAdapter;
use crate::common::connection_manager::ConnectionManager;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use axum_extra::headers::UserAgent;
use axum_extra::TypedHeader;
use axum_server::tls_rustls::RustlsConfig;
use bytes::{BufMut, BytesMut};
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::now_mills;
use common_config::broker::broker_config;
use common_metrics::network::record_ws_request_duration;
use futures_util::stream::StreamExt;
use kafka_protocol::messages::ResponseHeader;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
use protocol::kafka::packet::{KafkaHeader, KafkaPacketWrapper};
use protocol::mqtt::codec::MqttPacketWrapper;
use protocol::robust::{
    KafkaWrapperExtend, MqttWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::{self};
use tracing::{debug, error, info, warn};

pub const ROUTE_ROOT: &str = "/mqtt";

#[derive(Clone)]
pub struct WebSocketServerState {
    pub ws_port: u32,
    pub wss_port: u32,
    pub command: ArcCommandAdapter,
    pub connection_manager: Arc<ConnectionManager>,
    pub stop_sx: broadcast::Sender<bool>,
}

impl WebSocketServerState {
    pub fn new(
        ws_port: u32,
        wss_port: u32,
        command: ArcCommandAdapter,
        connection_manager: Arc<ConnectionManager>,
        stop_sx: broadcast::Sender<bool>,
    ) -> Self {
        Self {
            ws_port,
            wss_port,
            command,
            connection_manager,
            stop_sx,
        }
    }
}
#[derive(Clone)]
pub struct WebSocketServer {
    state: WebSocketServerState,
}

impl WebSocketServer {
    pub fn new(state: WebSocketServerState) -> Self {
        WebSocketServer { state }
    }

    pub async fn start_ws(&self) -> ResultCommonError {
        let ip: SocketAddr = format!("0.0.0.0:{}", self.state.ws_port).parse()?;
        let app = routes_v1(self.state.clone());

        info!("Broker WebSocket Server start success. addr:{}", ip);
        axum_server::bind(ip)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
        Ok(())
    }

    pub async fn start_wss(&self) -> ResultCommonError {
        let ip: SocketAddr = format!("0.0.0.0:{}", self.state.wss_port).parse()?;
        let app = routes_v1(self.state.clone());

        let config = broker_config();
        let tls_config = RustlsConfig::from_pem_file(
            PathBuf::from(config.runtime.tls_cert.clone()),
            PathBuf::from(config.runtime.tls_key.clone()),
        )
        .await?;

        info!("Broker WebSocket TLS Server start success. addr:{}", ip);
        axum_server::bind_rustls(ip, tls_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
        Ok(())
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
    let codec = RobustMQCodec::new();
    ws.protocols(["mqtt", "mqttv3.1"])
        .on_upgrade(move |socket| {
            handle_socket(
                socket,
                addr,
                state.command,
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
    mut codec: RobustMQCodec,
    connection_manager: Arc<ConnectionManager>,
    stop_sx: broadcast::Sender<bool>,
) {
    let (sender, mut receiver) = socket.split();
    let tcp_connection = NetworkConnection::new(NetworkConnectionType::WebSocket, addr, None);
    connection_manager.add_websocket_write(tcp_connection.connection_id, sender);
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
                                error!("Websocket failed to process protocol packet with error message :{e:?}");
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
                              connection_manager.close_connect(tcp_connection.connection_id).await;
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
    codec: &mut RobustMQCodec,
    command: ArcCommandAdapter,
    addr: SocketAddr,
    data: Vec<u8>,
) -> ResultCommonError {
    let receive_ms = now_mills();
    let mut buf = BytesMut::with_capacity(data.len());
    buf.put(data.as_slice());
    if let Some(packet) = codec.decode_data(&mut buf)? {
        info!("recv websocket packet:{packet:?}");

        let tcp_connection = if let Some(conn) = connection_manager.get_connect(connection_id) {
            conn
        } else {
            return Err(CommonError::NotFoundConnectionInCache(connection_id));
        };

        let robust_packet = match packet {
            RobustMQCodecWrapper::KAFKA(pkg) => RobustMQPacket::KAFKA(pkg.packet),
            RobustMQCodecWrapper::MQTT(pkg) => RobustMQPacket::MQTT(pkg.packet),
        };

        if let Some(resp_pkg) = command
            .apply(tcp_connection.clone(), addr, robust_packet)
            .await
        {
            // encode resp packet
            let mut response_buff = BytesMut::new();
            let resp_codec_wrapper = match resp_pkg.packet.clone() {
                RobustMQPacket::MQTT(pkg) => RobustMQCodecWrapper::MQTT(MqttPacketWrapper {
                    protocol_version: codec.mqtt_codec.protocol_version.unwrap(),
                    packet: pkg,
                }),
                RobustMQPacket::KAFKA(pkg) => RobustMQCodecWrapper::KAFKA(KafkaPacketWrapper {
                    api_version: 1,
                    header: KafkaHeader::Response(ResponseHeader::default()),
                    packet: pkg,
                }),
            };
            codec.encode_data(resp_codec_wrapper, &mut response_buff)?;

            // build resp wrapper
            let resp_wrapper = match resp_pkg.packet.clone() {
                RobustMQPacket::MQTT(pkg) => RobustMQPacketWrapper {
                    protocol: RobustMQProtocol::MQTT3,
                    extend: RobustMQWrapperExtend::MQTT(MqttWrapperExtend::default()),
                    packet: RobustMQPacket::MQTT(pkg),
                },
                RobustMQPacket::KAFKA(pkg) => RobustMQPacketWrapper {
                    protocol: RobustMQProtocol::KAFKA,
                    extend: RobustMQWrapperExtend::KAFKA(KafkaWrapperExtend::default()),
                    packet: RobustMQPacket::KAFKA(pkg),
                },
            };

            // write to client
            let response_ms = now_mills();
            if let Err(e) = connection_manager
                .write_websocket_frame(
                    tcp_connection.connection_id,
                    resp_wrapper,
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
            debug!("{}", "No backpacking is required for this request");
        }
    }

    Ok(())
}
