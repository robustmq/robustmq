use crate::handler::cache_manager::CacheManager;
use crate::handler::command::Command;
use crate::server::connection::NetworkConnection;
use crate::server::connection_manager::ConnectionManager;
use crate::subscribe::subscribe_cache::SubscribeCacheManager;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::Router;
use axum::{response::Response, routing::get};
use axum_extra::headers::UserAgent;
use axum_extra::TypedHeader;
use axum_server::tls_rustls::RustlsConfig;
use bytes::{BufMut, BytesMut};
use clients::poll::ClientPool;
use common_base::log::debug;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{error, info},
};
use futures_util::stream::StreamExt;
use protocol::mqtt::codec::{MQTTPacketWrapper, MqttCodec};
use protocol::mqtt::common::{MQTTPacket, MQTTProtocol};
use std::path::PathBuf;
use std::{net::SocketAddr, sync::Arc};
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self};

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
    info(format!(
        "Broker WebSocket Server start success. port:{}",
        config.mqtt.websocket_port
    ));
    match axum_server::bind(ip)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
    {
        Ok(()) => {}
        Err(e) => panic!("{}", e.to_string()),
    }
}

pub async fn websockets_server<S>(state: WebSocketServerState<S>)
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let config = broker_mqtt_conf();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.mqtt.websockets_port)
        .parse()
        .unwrap();
    let app = routes_v1(state);

    let tls_config = match RustlsConfig::from_pem_file(
        PathBuf::from(config.mqtt.tls_cert.clone()),
        PathBuf::from(config.mqtt.tls_key.clone()),
    )
    .await
    {
        Ok(cf) => cf,
        Err(e) => {
            panic!("{}", e.to_string());
        }
    };

    info(format!(
        "Broker WebSocket TLS Server start success. port:{}",
        config.mqtt.websockets_port
    ));
    match axum_server::bind_rustls(ip, tls_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
    {
        Ok(()) => {}
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
        state.sucscribe_manager.clone(),
        state.client_poll.clone(),
        state.connection_manager.clone(),
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
                state.stop_sx.clone(),
            )
        })
}

async fn handle_socket<S>(
    socket: WebSocket,
    addr: SocketAddr,
    mut command: Command<S>,
    mut codec: MqttCodec,
    connection_manager: Arc<ConnectionManager>,
    stop_sx: broadcast::Sender<bool>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let (sender, mut receiver) = socket.split();
    let tcp_connection = NetworkConnection::new(
        crate::server::connection::NetworkConnectionType::WebSocket,
        addr,
        None,
    );

    connection_manager.add_websocket_write(tcp_connection.connection_id, sender);
    connection_manager.add_connection(tcp_connection.clone());
    let mut protocol_version = MQTTProtocol::MQTT5;
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
            val = receiver.next()=>{
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
                                            packet.clone(),
                                        )
                                        .await
                                    {
                                        if let MQTTPacket::Connect(_,_,_,_,_,_) = packet {
                                            if let Some(pv) = connection_manager.get_connect_protocol(tcp_connection.connection_id){
                                                protocol_version = pv;
                                            }
                                        }

                                        let mut buff = BytesMut::new();
                                        let packet_wrapper = MQTTPacketWrapper {
                                            protocol_version: protocol_version.clone().into(),
                                            packet: resp_pkg,
                                        };

                                        info(format!("{packet_wrapper:?}"));
                                        match codec.encode_data(packet_wrapper, &mut buff){
                                            Ok(()) => {},
                                            Err(e) => {
                                                error(format!("Websocket encode back packet failed with error message: {e:?}"));
                                            }
                                        }
                                        match connection_manager.write_websocket_frame(tcp_connection.connection_id, Message::Binary(buf.to_vec())).await{
                                            Ok(()) => {},
                                            Err(e) => {
                                                error(format!("websocket returns failure to write the packet to the client with error message {e:?}"));
                                                connection_manager.clonse_connect(tcp_connection.connection_id).await;
                                                break;
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
