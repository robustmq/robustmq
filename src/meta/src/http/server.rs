use crate::{cluster::Cluster, storage::raft_core::RaftRocksDBStorageCore};
use axum::routing::get;
use axum::Router;
use common::{config::meta::MetaConfig, log::info};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use super::cluster::controller_index;

pub const ROUTE_ROOT: &str = "/";
pub const RAFT_INFO: &str = "/raft";

pub struct HttpServer {
    ip: SocketAddr,
    cluster: Arc<RwLock<Cluster>>,
    storage: Arc<RwLock<RaftRocksDBStorageCore>>,
}

impl HttpServer {
    pub fn new(
        config: MetaConfig,
        cluster: Arc<RwLock<Cluster>>,
        storage: Arc<RwLock<RaftRocksDBStorageCore>>,
    ) -> Self {
        let ip: SocketAddr = format!("{}:{}", config.addr, config.admin_port)
            .parse()
            .unwrap();
        return Self {
            ip,
            cluster,
            storage,
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
        let meta = Router::new().route(ROUTE_ROOT, get(move || controller_index(cluster)));

        let app = Router::new().merge(meta);
        return app;
    }
}
