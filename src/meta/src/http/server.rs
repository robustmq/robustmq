use super::meta::{storage, HttpMeta};
use crate::{
    cluster::Cluster,
    storage::{raft_core::RaftRocksDBStorageCore, rocksdb::RocksDBStorage},
};
use axum::routing::get;
use axum::Router;
use common::{config::meta::MetaConfig, log::info};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

pub const ROUTE_ROOT: &str = "/";

pub struct HttpServer {
    ip: SocketAddr,
    cluster: Arc<RwLock<Cluster>>,
    http_meta: HttpMeta,
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
        // let rds = RocksDBStorage::new(&config);
        let http_meta = HttpMeta::new();
        return Self {
            ip,
            cluster,
            http_meta,
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
        let meta = Router::new().route(ROUTE_ROOT, get(storage));
        let app = Router::new().merge(meta);
        return app;
    }
}
