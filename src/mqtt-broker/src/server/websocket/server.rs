use crate::handler::cache_manager::CacheManager;
use crate::subscribe::subscribe_cache::SubscribeCacheManager;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::Router;
use axum::{response::Response, routing::get};
use common_base::{config::broker_mqtt::broker_mqtt_conf, log::info};
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
    ws.protocols(["mqttv3.1"]).on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // let (mut sender, mut receiver) = socket.split();
    while let Some(msg) = socket.recv().await {
        info(format!("xxxxx:{:?}", msg));

        if socket
            .send(Message::Text("test".to_string()))
            .await
            .is_err()
        {
            // client disconnected
            return;
        }
    }
}
