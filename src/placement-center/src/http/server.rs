use crate::{cache::{engine_cluster::EngineClusterCache, placement_cluster::PlacementClusterCache}, rocksdb::{raft::RaftMachineStorage, rwlayer::RwLayer}};
use axum::routing::get;
use axum::Router;
use common::log::info;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use super::cluster::{metrics, placement_center, storage_engine, test};

pub const ROUTE_ROOT: &str = "/";
pub const STORAGE_ENGINE: &str = "/storage-engine";
pub const ROUTE_TEST: &str = "/test";
pub const ROUTE_METRICS: &str = "/metrics";

#[derive(Clone)]
pub struct HttpServerState {
    pub placement_storage: Arc<RwLock<PlacementClusterCache>>,
    pub raft_storage: Arc<RwLock<RaftMachineStorage>>,
    pub cluster_storage: Arc<RwLayer>,
    pub engine_cluster: Arc<RwLock<EngineClusterCache>>,
}

impl HttpServerState {
    pub fn new(
        placement_storage: Arc<RwLock<PlacementClusterCache>>,
        raft_storage: Arc<RwLock<RaftMachineStorage>>,
        cluster_storage: Arc<RwLayer>,
        engine_cluster: Arc<RwLock<EngineClusterCache>>,
    ) -> Self {
        return Self {
            placement_storage,
            raft_storage,
            cluster_storage,
            engine_cluster,
        };
    }
}

pub async fn start_http_server(port: u16, state: HttpServerState) {
    let ip: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
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
        .route(ROUTE_TEST, get(test))
        .route(ROUTE_METRICS, get(metrics))
        ;
    let app = Router::new().merge(meta);
    return app.with_state(state);
}
