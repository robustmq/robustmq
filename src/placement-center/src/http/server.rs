use crate::{
    cluster::PlacementCluster,
    storage::{cluster_storage::ClusterStorage, raft_core::RaftRocksDBStorageCore},
    storage_cluster::StorageCluster,
};
use axum::routing::get;
use axum::Router;
use common::log::info;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use super::cluster::{placement_center, storage_engine};

pub const ROUTE_ROOT: &str = "/";
pub const STORAGE_ENGINE: &str = "/storage-engine";

#[derive(Clone)]
pub struct HttpServerState {
    pub placement_storage: Arc<RwLock<PlacementCluster>>,
    pub raft_storage: Arc<RwLock<RaftRocksDBStorageCore>>,
    pub cluster_storage: Arc<ClusterStorage>,
    pub engine_cluster: Arc<RwLock<StorageCluster>>,
}

impl HttpServerState {
    pub fn new(
        placement_storage: Arc<RwLock<PlacementCluster>>,
        raft_storage: Arc<RwLock<RaftRocksDBStorageCore>>,
        cluster_storage: Arc<ClusterStorage>,
        engine_cluster: Arc<RwLock<StorageCluster>>,
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
        .route(STORAGE_ENGINE, get(storage_engine));
    let app = Router::new().merge(meta);
    return app.with_state(state);
}
