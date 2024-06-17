use super::services::GrpcBrokerServices;
use crate::core::metadata_cache::MetadataCacheManager;
use clients::poll::ClientPool;
use common_base::log::info;
use protocol::broker_server::generate::mqtt::mqtt_broker_service_server::MqttBrokerServiceServer;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tonic::transport::Server;

pub struct GrpcServer<T> {
    port: u32,
    metadata_cache: Arc<MetadataCacheManager>,
    metadata_storage_adapter: Arc<T>,
    client_poll: Arc<ClientPool>,
}

impl<T> GrpcServer<T>
where
    T: StorageAdapter + Send + Sync + 'static,
{
    pub fn new(
        port: u32,
        metadata_cache: Arc<MetadataCacheManager>,
        metadata_storage_adapter: Arc<T>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        return Self {
            port,
            metadata_cache,
            metadata_storage_adapter,
            client_poll,
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
            self.client_poll.clone(),
        );
        Server::builder()
            .add_service(MqttBrokerServiceServer::new(service_handler))
            .serve(addr)
            .await
            .unwrap();
    }
}
