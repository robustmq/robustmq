use crate::{
    cluster::PlacementCluster,
    storage::{cluster_storage::ClusterStorage, raft_core::RaftRocksDBStorageCore}, storage_cluster::StorageCluster,
};
use axum::routing::get;
use axum::Router;
use common::{config::placement_center::PlacementCenterConfig, log::info};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use super::cluster::{controller_index, storage_engine};

pub const ROUTE_ROOT: &str = "/";
pub const STORAGE_ENGINE: &str = "/storage-engine";

pub struct HttpServer {
    ip: SocketAddr,
    cluster: Arc<RwLock<PlacementCluster>>,
    storage: Arc<RwLock<RaftRocksDBStorageCore>>,
    cluster_storage: Arc<ClusterStorage>,
    engine_cluster: Arc<RwLock<StorageCluster>>,
}

impl HttpServer {
    pub fn new(
        config: PlacementCenterConfig,
        cluster: Arc<RwLock<PlacementCluster>>,
        storage: Arc<RwLock<RaftRocksDBStorageCore>>,
        cluster_storage: Arc<ClusterStorage>,
        engine_cluster: Arc<RwLock<StorageCluster>>,
    ) -> Self {
        let ip: SocketAddr = format!("{}:{}", config.addr, config.http_port)
            .parse()
            .unwrap();
        return Self {
            ip,
            cluster,
            storage,
            cluster_storage,
            engine_cluster
        };
    }

    pub async fn start(&self) {
        let ip = self.ip.clone();
        let app = self.routes();
        let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
        info(&format!(
            "RobustMQ Meta HTTP Server start success. bind addr:{}",
            ip
        ));
        axum::serve(listener, app).await.unwrap();
    }

    pub fn routes(&self) -> Router {
        let cluster = self.cluster.clone();
        let storage = self.storage.clone();
        let cluster_storage = self.cluster_storage.clone();
        let engine_cluster = self.engine_cluster.clone();
        let meta = Router::new()
            .route(ROUTE_ROOT, get(move || controller_index(cluster, storage)))
            .route(STORAGE_ENGINE, get(|| storage_engine(engine_cluster)));

        let app = Router::new().merge(meta);
        return app;
    }
}
