use std::sync::Arc;

use crate::metadata::cache::MetadataCache;

use super::services::GrpcBrokerServices;
use common_base::log::info;
use protocol::broker_server::generate::mqtt::mqtt_broker_service_server::MqttBrokerServiceServer;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::RwLock;
use tonic::transport::Server;

pub struct GrpcServer<T> {
    port: u32,
    metadata_cache: Arc<RwLock<MetadataCache<T>>>,
    metadata_storage_adapter: Arc<T>,
}

impl<T> GrpcServer<T>
where
    T: StorageAdapter + Send + Sync + 'static,
{
    pub fn new(
        port: u32,
        metadata_cache: Arc<RwLock<MetadataCache<T>>>,
        metadata_storage_adapter: Arc<T>,
    ) -> Self {
        return Self {
            port,
            metadata_cache,
            metadata_storage_adapter,
        };
    }
    pub async fn start(&self) {
        let addr = format!("0.0.0.0:{}", self.port).parse().unwrap();
        info(format!(
            "Broker Grpc Server start success. bind addr:{}",
            addr
        ));
        let service_handler = GrpcBrokerServices::new(
            self.metadata_cache.clone(),
            self.metadata_storage_adapter.clone(),
        );
        Server::builder()
            .add_service(MqttBrokerServiceServer::new(service_handler))
            .serve(addr)
            .await
            .unwrap();
    }
}
