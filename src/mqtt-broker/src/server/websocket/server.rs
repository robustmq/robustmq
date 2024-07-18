use crate::handler::cache_manager::{self, CacheManager};
use crate::handler::command::Command;
use crate::server::tcp::connection::NetworkConnection;
use crate::server::tcp::connection_manager::ConnectionManager;
use crate::server::tcp::packet::{RequestPackage, ResponsePackage};
use crate::subscribe::subscribe_cache::SubscribeCacheManager;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::Router;
use axum::{response::Response, routing::get};
use axum_extra::headers::UserAgent;
use axum_extra::TypedHeader;
use bytes::{BufMut, BytesMut};
use clients::poll::ClientPool;
use common_base::log::debug;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{error, info},
};
use protocol::mqtt::codec::{MQTTPacketWrapper, MqttCodec};
use protocol::mqtt::common::MQTTProtocol;
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;
pub const ROUTE_ROOT: &str = "/mqtt";

#[derive(Clone)]
pub struct WebSocketServerState<S> {
    sucscribe_manager: Arc<SubscribeCacheManager>,
    cache_manager: Arc<CacheManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
}

impl<S> WebSocketServerState<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        sucscribe_manager: Arc<SubscribeCacheManager>,
        cache_manager: Arc<CacheManager>,
        connection_manager: Arc<ConnectionManager>,
        message_storage_adapter: Arc<S>,
        client_poll: Arc<ClientPool>,
        stop_sx: broadcast::Sender<bool>,
    ) -> Self {
        return Self {
            sucscribe_manager,
            cache_manager,
            connection_manager,
            message_storage_adapter,
            client_poll,
            stop_sx,
        };
    }
}

pub async fn websocket_server<S>(state: WebSocketServerState<S>)
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let config = broker_mqtt_conf();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.mqtt.websocket_port)
        .parse()
        .unwrap();
    let app = routes_v1(state);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    match axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        Ok(()) => {
            info(format!(
                "Broker WebSocket Server start success. bind addr:{}",
                ip
            ));
        }
        Err(e) => panic!("{}", e.to_string()),
    }
}

fn routes_v1<S>(state: WebSocketServerState<S>) -> Router
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let mqtt_ws = Router::new().route(ROUTE_ROOT, get(ws_handler));

    let app = Router::new().merge(mqtt_ws);
    return app.with_state(state);
}

async fn ws_handler<S>(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketServerState<S>>,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown Source")
    };
    info(format!("`{user_agent}` at {addr} connected."));
    let command = Command::new(
        state.cache_manager.clone(),
        state.message_storage_adapter.clone(),
        state.response_queue_sx.clone(),
        state.sucscribe_manager.clone(),
        state.client_poll.clone(),
        state.stop_sx.clone(),
    );
    let codec = MqttCodec::new(None);
    ws.protocols(["mqtt", "mqttv3.1"])
        .on_upgrade(move |socket| {
            handle_socket(
                socket,
                addr,
                command,
                codec,
                state.connection_manager.clone(),
                state.cache_manager.clone(),
                state.stop_sx.clone(),
            )
        })
}

async fn handle_socket<S>(
    mut socket: WebSocket,
    addr: SocketAddr,
    mut command: Command<S>,
    mut codec: MqttCodec,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    stop_sx: broadcast::Sender<bool>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let tcp_connection = NetworkConnection::new(
        crate::server::tcp::connection::NetworkConnectionType::WebSocket,
        addr,
        None,
    );
    connection_manager.add(tcp_connection.clone());
    let protocol_version = MQTTProtocol::MQTT5.into();
    let mut stop_rx = stop_sx.subscribe();
    loop {
        select! {
            val = stop_rx.recv() =>{
                match val{
                    Ok(flag) => {
                        if flag {
                            break;
                        }
                    }
                    Err(_) => {}
                }
            },
            val = socket.recv()=>{
                if let Some(msg) = val{
                    match msg {
                        Ok(Message::Binary(data)) => {
                            let mut buf = BytesMut::with_capacity(data.len());
                            buf.put(data.as_slice());
                            match codec.decode_data(&mut buf) {
                                Ok(Some(packet)) => {
                                    info(format!("recv websocket packet:{packet:?}"));
                                    if let Some(resp_pkg) = command
                                        .apply(
                                            connection_manager.clone(),
                                            tcp_connection.clone(),
                                            addr.clone(),
                                            packet,
                                        )
                                        .await
                                    {
                                        let mut buff = BytesMut::new();
                                        let packet_wrapper = MQTTPacketWrapper {
                                            protocol_version,
                                            packet: resp_pkg,
                                        };
                                        codec.encode_data(packet_wrapper, &mut buff);

                                        let mut times = 0;
                                        let cluster = cache_manager.get_cluster_info();
                                        loop {
                                            match socket.send(Message::Binary(buf.to_vec())).await {
                                                Ok(()) => {}
                                                Err(e) => {
                                                    error(format!("Failed to write data back to Websocket client with error message :{e:?}"));
                                                    if times > cluster.send_max_try_mut_times {
                                                        break;
                                                    }
                                                    sleep(Duration::from_millis(
                                                        cluster.send_try_mut_sleep_time_ms,
                                                    ))
                                                    .await;
                                                    times = times + 1;
                                                }
                                            }
                                        }
                                    }
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    error(format!("Websocket failed to parse MQTT protocol packet with error message :{e:?}"));
                                }
                            }
                        }
                        Ok(Message::Text(data)) => {
                            debug(format!(
                                "websocket server receives a TEXT message with the following content: {data}"
                            ));
                        }
                        Ok(Message::Ping(data)) => {
                            debug(format!(
                                "websocket server receives a Ping message with the following content: {data:?}"
                            ));
                        }
                        Ok(Message::Pong(data)) => {
                            debug(format!(
                                "websocket server receives a Pong message with the following content: {data:?}"
                            ));
                        }
                        Ok(Message::Close(data)) => {
                            if let Some(cf) = data {
                                info(format!(
                                    ">>> {} sent close with code {} and reason `{}`",
                                    addr, cf.code, cf.reason
                                ));
                            } else {
                                info(format!(
                                    ">>> {addr} somehow sent close message without CloseFrame"
                                ));
                            }
                            socket.close();
                            break;
                        }
                        Err(e) => error(format!(
                            "websocket server parsing request packet error, error message :{e:?}"
                        )),
                    }
                }
            }
        }
    }
}
