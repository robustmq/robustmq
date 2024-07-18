use crate::handler::cache_manager::CacheManager;
use crate::handler::command::Command;
use crate::server::tcp::packet::{RequestPackage, ResponsePackage};
use crate::subscribe::subscribe_cache::SubscribeCacheManager;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, WebSocketUpgrade};
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
use protocol::mqtt::codec::MqttCodec;
use std::{net::SocketAddr, sync::Arc};
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::{self, Sender};
pub const ROUTE_ROOT: &str = "/mqtt";

#[derive(Clone)]
pub struct WebSocketServerState<S> {
    sucscribe_manager: Arc<SubscribeCacheManager>,
    cache_manager: Arc<CacheManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
    request_queue_sx: Sender<RequestPackage>,
    response_queue_sx: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
}

impl<S> WebSocketServerState<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        sucscribe_manager: Arc<SubscribeCacheManager>,
        cache_manager: Arc<CacheManager>,
        message_storage_adapter: Arc<S>,
        client_poll: Arc<ClientPool>,
        request_queue_sx: Sender<RequestPackage>,
        response_queue_sx: Sender<ResponsePackage>,
        stop_sx: broadcast::Sender<bool>,
    ) -> Self {
        return Self {
            sucscribe_manager,
            cache_manager,
            message_storage_adapter,
            client_poll,
            request_queue_sx,
            response_queue_sx,
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

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown Source")
    };
    info(format!("`{user_agent}` at {addr} connected."));
    let command = Command::new(
        cache_manager.clone(),
        message_storage_adapter.clone(),
        response_queue_sx.clone(),
        sucscribe_manager.clone(),
        client_poll.clone(),
        stop_sx.clone(),
    );

    ws.protocols(["mqtt", "mqttv3.1"])
        .on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr) {
    // let (mut sender, mut receiver) = socket.split();
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Binary(data)) => {
                let mut buf = BytesMut::with_capacity(data.len());
                buf.put(data.as_slice());
                let mut codec = MqttCodec::new(None);
                let res = codec.decode_data(&mut buf);
                info(format!("Binary:{},{:?}", data.len(), res));
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
