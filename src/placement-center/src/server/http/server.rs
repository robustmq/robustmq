use super::index::{index, metrics};
use super::mqtt::mqtt_routes;
use crate::raft::metadata::RaftGroupMetadata;
use crate::{
    cache::{journal::JournalCacheManager, placement::PlacementCacheManager},
    storage::placement::raft::RaftMachineStorage,
};
use axum::routing::get;
use axum::Router;
use clients::mqtt;
use common_base::{config::placement_center::placement_center_conf, log::info_meta};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

pub const ROUTE_ROOT: &str = "/";
pub const ROUTE_METRICS: &str = "/metrics";

#[derive(Clone)]
pub struct HttpServerState {
    pub placement_cache: Arc<RwLock<RaftGroupMetadata>>,
    pub raft_storage: Arc<RwLock<RaftMachineStorage>>,
    pub cluster_cache: Arc<PlacementCacheManager>,
    pub engine_cache: Arc<JournalCacheManager>,
}

impl HttpServerState {
    pub fn new(
        placement_cache: Arc<RwLock<RaftGroupMetadata>>,
        raft_storage: Arc<RwLock<RaftMachineStorage>>,
        cluster_cache: Arc<PlacementCacheManager>,
        engine_cache: Arc<JournalCacheManager>,
    ) -> Self {
        return Self {
            placement_cache,
            raft_storage,
            cluster_cache,
            engine_cache,
        };
    }
}

pub async fn start_http_server(state: HttpServerState) {
    let config = placement_center_conf();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.http_port).parse().unwrap();
    let app = routes(state);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    info_meta(&format!(
        "Placement Center HTTP Server start success. bind addr:{}",
        ip
    ));
    axum::serve(listener, app).await.unwrap();
}

fn routes(state: HttpServerState) -> Router {
    let common = Router::new()
        .route(ROUTE_ROOT, get(index))
        .route(ROUTE_METRICS, get(metrics));

    let mqtt = mqtt_routes();

    let app = Router::new().merge(common).merge(mqtt);
    return app.with_state(state);
}
