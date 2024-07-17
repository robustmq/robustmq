use crate::handler::cache_manager::CacheManager;
use crate::subscribe::subscribe_cache::SubscribeCacheManager;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::Router;
use axum::{response::Response, routing::get};
use bytes::{BufMut, BytesMut};
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    log::{error, info},
};
use protocol::mqtt::codec::MqttCodec;
use std::{net::SocketAddr, sync::Arc};

pub const ROUTE_ROOT: &str = "/mqtt";

#[derive(Clone)]
pub struct WebSocketServerState {
    pub cache_metadata: Arc<CacheManager>,
    pub subscribe_cache: Arc<SubscribeCacheManager>,
}

impl WebSocketServerState {
    pub fn new(
        cache_metadata: Arc<CacheManager>,
        subscribe_cache: Arc<SubscribeCacheManager>,
    ) -> Self {
        return Self {
            cache_metadata,
            subscribe_cache,
        };
    }
}

pub async fn websocket_server(state: WebSocketServerState) {
    let config = broker_mqtt_conf();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.mqtt.websocket_port)
        .parse()
        .unwrap();
    let app = routes_v1(state);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    info(format!(
        "Broker WebSocket Server start success. bind addr:{}",
        ip
    ));
    axum::serve(listener, app).await.unwrap();
}

fn routes_v1(state: WebSocketServerState) -> Router {
    let mqtt_ws = Router::new().route(ROUTE_ROOT, get(handler));

    let app = Router::new().merge(mqtt_ws);
    return app.with_state(state);
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.protocols(["mqtt", "mqttv3.1"]).on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
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
                info(format!("Text:{},{:?}", data.len(), data));
            }
            Ok(Message::Ping(data)) => {
                info(format!("Ping:{},{:?}", data.len(), data));
            }
            Ok(Message::Pong(data)) => {
                info(format!("Pong:{},{:?}", data.len(), data));
            }
            Ok(Message::Close(data)) => {}
            Err(e) => error(e.to_string()),
        }
    }
}
