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
use crate::common::channel::RequestChannel;
use crate::common::connection_manager::ConnectionManager;
use crate::common::handler::handler_process;
use crate::common::packet::RequestPackage;
use crate::context::ProcessorConfig;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use axum_extra::headers::UserAgent;
use axum_extra::TypedHeader;
use axum_server::tls_rustls::RustlsConfig;
use bytes::{BufMut, BytesMut};
use common_base::error::ResultCommonError;
use common_config::broker::broker_config;
use futures_util::stream::StreamExt;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

pub const ROUTE_ROOT: &str = "/mqtt";

#[derive(Clone)]
pub struct WebSocketServerState {
    pub ws_port: u32,
    pub wss_port: u32,
    pub command: ArcCommandAdapter,
    pub connection_manager: Arc<ConnectionManager>,
    pub stop_sx: broadcast::Sender<bool>,
    pub request_channel: Arc<RequestChannel>,
    pub proc_config: ProcessorConfig,
}

impl WebSocketServerState {
    pub fn new(
        ws_port: u32,
        wss_port: u32,
        command: ArcCommandAdapter,
        connection_manager: Arc<ConnectionManager>,
        stop_sx: broadcast::Sender<bool>,
        proc_config: ProcessorConfig,
    ) -> Self {
        let request_channel = Arc::new(RequestChannel::new(proc_config.channel_size));
        Self {
            ws_port,
            wss_port,
            command,
            connection_manager,
            stop_sx,
            request_channel,
            proc_config,
        }
    }
}

#[derive(Clone)]
pub struct WebSocketServer {
    name: String,
    state: WebSocketServerState,
}

impl WebSocketServer {
    pub fn new(name: String, state: WebSocketServerState) -> Self {
        WebSocketServer { name, state }
    }

    pub async fn start_ws(&self) -> ResultCommonError {
        self.start_handlers();

        let ip: SocketAddr = format!("0.0.0.0:{}", self.state.ws_port).parse()?;
        let app = routes_v1(self.state.clone());

        info!("{} WebSocket Server start success. addr:{}", self.name, ip);
        axum_server::bind(ip)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
        Ok(())
    }

    pub async fn start_wss(&self) -> ResultCommonError {
        self.start_handlers();

        let ip: SocketAddr = format!("0.0.0.0:{}", self.state.wss_port).parse()?;
        let app = routes_v1(self.state.clone());

        let config = broker_config();
        let tls_config = RustlsConfig::from_pem_file(
            PathBuf::from(config.runtime.tls_cert.clone()),
            PathBuf::from(config.runtime.tls_key.clone()),
        )
        .await?;

        info!(
            "{} WebSocket TLS Server start success. addr:{}",
            self.name, ip
        );
        axum_server::bind_rustls(ip, tls_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
        Ok(())
    }

    fn start_handlers(&self) {
        handler_process(
            self.state.proc_config.handler_process_num,
            self.state.connection_manager.clone(),
            self.state.command.clone(),
            self.state.request_channel.clone(),
            NetworkConnectionType::WebSocket,
            self.state.stop_sx.clone(),
        );
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

    debug!("websocket `{user_agent}` at {addr} connected.");
    ws.protocols(["mqtt", "mqttv3.1"])
        .on_upgrade(move |socket| {
            handle_socket(
                socket,
                addr,
                state.connection_manager.clone(),
                state.request_channel.clone(),
                state.stop_sx.clone(),
            )
        })
}

async fn handle_socket(
    socket: WebSocket,
    addr: SocketAddr,
    connection_manager: Arc<ConnectionManager>,
    request_channel: Arc<RequestChannel>,
    stop_sx: broadcast::Sender<bool>,
) {
    let (sender, mut receiver) = socket.split();
    let connection = NetworkConnection::new(NetworkConnectionType::WebSocket, addr, None);
    let connection_id = connection.connection_id;
    connection_manager.add_websocket_write(connection_id, sender);
    connection_manager.add_connection(connection);
    let mut stop_rx = stop_sx.subscribe();
    let mut codec = RobustMQCodec::new();

    loop {
        select! {
            val = stop_rx.recv() => {
                if let Ok(true) = val {
                    break;
                }
            },
            val = receiver.next() => {
                if let Some(msg) = val {
                    match msg {
                        Ok(Message::Binary(data)) => {
                            let mut buf = BytesMut::with_capacity(data.len());
                            buf.put(data.as_ref());
                            match codec.decode_data(&mut buf) {
                                Ok(Some(packet)) => {
                                    let robust_packet = match packet {
                                        RobustMQCodecWrapper::MQTT(pkg) => RobustMQPacket::MQTT(pkg.packet),
                                        RobustMQCodecWrapper::KAFKA(pkg) => RobustMQPacket::KAFKA(pkg.packet),
                                        RobustMQCodecWrapper::StorageEngine(pkg) => RobustMQPacket::StorageEngine(pkg),
                                    };
                                    let package = RequestPackage::new(connection_id, addr, robust_packet);
                                    request_channel.send(package).await;
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    error!("WebSocket decode error: {:?}", e);
                                }
                            }
                        }
                        Ok(Message::Text(data)) => {
                            warn!("websocket server receives a TEXT message: {data}");
                        }
                        Ok(Message::Ping(data)) => {
                            debug!("websocket server receives a Ping message: {data:?}");
                        }
                        Ok(Message::Pong(data)) => {
                            debug!("websocket server receives a Pong message: {data:?}");
                        }
                        Ok(Message::Close(data)) => {
                            if let Some(cf) = data {
                                info!(">>> {} sent close with code {} and reason `{}`", addr, cf.code, cf.reason);
                            } else {
                                info!(">>> {addr} sent close without CloseFrame");
                            }
                            connection_manager.mark_close_connect(connection_id).await;
                            break;
                        }
                        Err(e) => {
                            warn!("websocket server parsing request packet error: {e:?}");
                            connection_manager.mark_close_connect(connection_id).await;
                            break;
                        },
                    }
                } else {
                    debug!("WebSocket connection closed (EOF): connection_id={}", connection_id);
                    connection_manager.mark_close_connect(connection_id).await;
                    break;
                }
            }
        }
    }
}
