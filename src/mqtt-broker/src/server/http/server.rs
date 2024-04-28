use super::node::{hearbeat_info, index, metadata_info, metrics, subscribe_info};
use crate::{
    core::heartbeat_manager::HeartbeatManager, metadata::cache::MetadataCacheManager,
    server::tcp::packet::ResponsePackage, subscribe::manager::SubScribeManager,
};
use axum::routing::get;
use axum::Router;
use common_base::{config::broker_mqtt::broker_mqtt_conf, log::info};
use std::{net::SocketAddr, sync::Arc};
use storage_adapter::storage::StorageAdapter;
use tokio::sync::broadcast::Sender;

pub const ROUTE_ROOT: &str = "/";
pub const ROUTE_HEARTBEAT_INFO: &str = "/heartbeat-info";
pub const ROUTE_METADATA_INFO: &str = "/metadata-info";
pub const ROUTE_SUBSCRIBE_INFO: &str = "/subscribe-info";
pub const ROUTE_METRICS: &str = "/metrics";

#[derive(Clone)]
pub struct HttpServerState<T> {
    pub metadata_cache: Arc<MetadataCacheManager<T>>,
    pub heartbeat_manager: Arc<HeartbeatManager>,
    pub subscribe_manager: Arc<SubScribeManager<T>>,
    pub response_queue_sx4: Sender<ResponsePackage>,
    pub response_queue_sx5: Sender<ResponsePackage>,
}

impl<T> HttpServerState<T>
where
    T: StorageAdapter,
{
    pub fn new(
        metadata_cache: Arc<MetadataCacheManager<T>>,
        heartbeat_manager: Arc<HeartbeatManager>,
        subscribe_manager: Arc<SubScribeManager<T>>,
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

pub async fn start_http_server<T>(state: HttpServerState<T>)
where
    T: Clone + Send + Sync + 'static,
{
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

fn routes<T>(state: HttpServerState<T>) -> Router
where
    T: Clone + Send + Sync + 'static,
{
    let meta = Router::new()
        .route(ROUTE_ROOT, get(index))
        .route(ROUTE_HEARTBEAT_INFO, get(hearbeat_info))
        .route(ROUTE_METADATA_INFO, get(metadata_info))
        .route(ROUTE_SUBSCRIBE_INFO, get(subscribe_info))
        .route(ROUTE_METRICS, get(metrics));
    let app = Router::new().merge(meta);
    return app.with_state(state);
}
