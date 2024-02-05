use crate::{
    cluster::Cluster,
    storage::{cluster_storage::ClusterStorage, raft_core::RaftRocksDBStorageCore},
};
use axum::routing::get;
use axum::Router;
use common::{config::meta::MetaConfig, log::info};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use super::cluster::{cluster_info, controller_index};

pub const ROUTE_ROOT: &str = "/";
pub const CLUSTER_INFO: &str = "/cluster-info";

pub struct HttpServer {
    ip: SocketAddr,
    cluster: Arc<RwLock<Cluster>>,
    storage: Arc<RwLock<RaftRocksDBStorageCore>>,
    cluster_storage: Arc<RwLock<ClusterStorage>>,
}

impl HttpServer {
    pub fn new(
        config: MetaConfig,
        cluster: Arc<RwLock<Cluster>>,
        storage: Arc<RwLock<RaftRocksDBStorageCore>>,
        cluster_storage: Arc<RwLock<ClusterStorage>>,
    ) -> Self {
        let ip: SocketAddr = format!("{}:{}", config.addr, config.admin_port)
            .parse()
            .unwrap();
        return Self {
            ip,
            cluster,
            storage,
            cluster_storage,
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
        let meta = Router::new()
            .route(ROUTE_ROOT, get(move || controller_index(cluster, storage)))
            .route(CLUSTER_INFO, get(|| cluster_info(cluster_storage)));

        let app = Router::new().merge(meta);
        return app;
    }
}
