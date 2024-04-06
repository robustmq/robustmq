use std::sync::Arc;

use crate::metadata::cache::MetadataCache;

use super::services::GrpcBrokerServices;
use common_base::log::info;
use protocol::broker_server::generate::mqtt::mqtt_broker_service_server::MqttBrokerServiceServer;
use tokio::sync::RwLock;
use tonic::transport::Server;

pub struct GrpcServer {
    port: usize,
    metadata_cache: Arc<RwLock<MetadataCache>>,
}

impl GrpcServer {
    pub fn new(port: usize, metadata_cache: Arc<RwLock<MetadataCache>>) -> Self {
        return Self {
            port,
            metadata_cache,
        };
    }
    pub async fn start(&self) {
        let addr = format!("0.0.0.0:{}", self.port).parse().unwrap();
        info(format!(
            "Broker Grpc Server start success. bind addr:{}",
            addr
        ));
        let service_handler = GrpcBrokerServices::new(self.metadata_cache.clone());
        Server::builder()
            .add_service(MqttBrokerServiceServer::new(service_handler))
            .serve(addr)
            .await
            .unwrap();
    }
}
