use axum::routing::get;
use axum::Router;
use common_base::{config::broker_mqtt::broker_mqtt_conf, log::info};
use flume::Sender;
use tokio::sync::RwLock;
use std::{
    net::SocketAddr,
    sync::Arc,
};
use crate::{
    cluster::heartbeat_manager::HeartbeatManager, metadata::cache::MetadataCache,
    server::tcp::packet::ResponsePackage, subscribe::manager::SubScribeManager,
};
use super::node::{
    hearbeat_info, index, metadata_info, metrics, subscribe_info, test_subscribe_pub,
};

pub const ROUTE_ROOT: &str = "/";
pub const ROUTE_HEARTBEAT_INFO: &str = "/heartbeat-info";
pub const ROUTE_METADATA_INFO: &str = "/metadata-info";
pub const ROUTE_SUBSCRIBE_INFO: &str = "/subscribe-info";
pub const ROUTE_TEST_SUBSCRIBE_PUB: &str = "/test-subscribe-pub";
// pub const ROUTE_TEST_SUBSCRIBE_PUB: &str = "/test-subscribe-pub";
pub const ROUTE_METRICS: &str = "/metrics";

#[derive(Clone)]
pub struct HttpServerState {
    pub metadata_cache: Arc<RwLock<MetadataCache>>,
    pub heartbeat_manager: Arc<RwLock<HeartbeatManager>>,
    pub subscribe_manager: Arc<RwLock<SubScribeManager>>,
    pub response_queue_sx4: Sender<ResponsePackage>,
    pub response_queue_sx5: Sender<ResponsePackage>,
}

impl HttpServerState {
    pub fn new(
        metadata_cache: Arc<RwLock<MetadataCache>>,
        heartbeat_manager: Arc<RwLock<HeartbeatManager>>,
        subscribe_manager: Arc<RwLock<SubScribeManager>>,
        response_queue_sx4: Sender<ResponsePackage>,
        response_queue_sx5: Sender<ResponsePackage>,
    ) -> Self {
        return Self {
            metadata_cache,
            heartbeat_manager,
            subscribe_manager,
            response_queue_sx4,
            response_queue_sx5,
        };
    }
}

pub async fn start_http_server(state: HttpServerState) {
    let config = broker_mqtt_conf();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.http_port).parse().unwrap();
    let app = routes(state);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    info(format!(
        "Broker HTTP Server start success. bind addr:{}",
        ip
    ));
    axum::serve(listener, app).await.unwrap();
}

fn routes(state: HttpServerState) -> Router {
    let meta = Router::new()
        .route(ROUTE_ROOT, get(index))
        .route(ROUTE_HEARTBEAT_INFO, get(hearbeat_info))
        .route(ROUTE_METADATA_INFO, get(metadata_info))
        .route(ROUTE_SUBSCRIBE_INFO, get(subscribe_info))
        .route(ROUTE_TEST_SUBSCRIBE_PUB, get(test_subscribe_pub))
        .route(ROUTE_METRICS, get(metrics));
    let app = Router::new().merge(meta);
    return app.with_state(state);
}
