use super::{
    placement_center::{metrics, placement_center},
    storage_engine::{shard_info, shard_list, storage_engine},
};
use crate::{
    cache::{engine::EngineClusterCache, placement::PlacementClusterCache},
    storage::raft::RaftMachineStorage,
};
use axum::routing::get;
use axum::Router;
use common::{config::placement_center::placement_center_conf, log::info};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

pub const ROUTE_ROOT: &str = "/";
pub const STORAGE_ENGINE: &str = "/storage-engine";
pub const STORAGE_ENGINE_SHARD_LIST: &str = "/storage-engine/shard-list";
pub const STORAGE_ENGINE_SHARD_INFO: &str = "/storage-engine/shard-info";
pub const ROUTE_METRICS: &str = "/metrics";

#[derive(Clone)]
pub struct HttpServerState {
    pub placement_cache: Arc<RwLock<PlacementClusterCache>>,
    pub raft_storage: Arc<RwLock<RaftMachineStorage>>,
    pub engine_cache: Arc<RwLock<EngineClusterCache>>,
}

impl HttpServerState {
    pub fn new(
        placement_cache: Arc<RwLock<PlacementClusterCache>>,
        raft_storage: Arc<RwLock<RaftMachineStorage>>,
        engine_cache: Arc<RwLock<EngineClusterCache>>,
    ) -> Self {
        return Self {
            placement_cache,
            raft_storage,
            engine_cache,
        };
    }
}

pub async fn start_http_server(state: HttpServerState) {
    let config = placement_center_conf();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.http_port).parse().unwrap();
    let app = routes(state);
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    info(&format!(
        "RobustMQ Meta HTTP Server start success. bind addr:{}",
        ip
    ));
    axum::serve(listener, app).await.unwrap();
}

fn routes(state: HttpServerState) -> Router {
    let meta = Router::new()
        .route(ROUTE_ROOT, get(placement_center))
        .route(STORAGE_ENGINE, get(storage_engine))
        .route(STORAGE_ENGINE_SHARD_LIST, get(shard_list))
        .route(STORAGE_ENGINE_SHARD_INFO, get(shard_info))
        .route(ROUTE_METRICS, get(metrics));
    let app = Router::new().merge(meta);
    return app.with_state(state);
}
