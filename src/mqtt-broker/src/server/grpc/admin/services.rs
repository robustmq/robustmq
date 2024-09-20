use crate::{handler::cache::CacheManager, subscribe::subscribe_manager::SubscribeManager};
use clients::poll::ClientPool;
use protocol::broker_server::generate::admin::{
    mqtt_broker_admin_service_server::MqttBrokerAdminService, StatusReply, StatusRequest,
};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tonic::{Request, Response, Status};

pub struct GrpcAdminServices<S> {
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_poll: Arc<ClientPool>,
    message_storage_adapter: Arc<S>,
}

impl<S> GrpcAdminServices<S> {
    pub fn new(
        metadata_cache: Arc<CacheManager>,
        subscribe_manager: Arc<SubscribeManager>,
        client_poll: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
    ) -> Self {
        return GrpcAdminServices {
            cache_manager: metadata_cache,
            subscribe_manager,
            client_poll,
            message_storage_adapter,
        };
    }
}

#[tonic::async_trait]
impl<S> MqttBrokerAdminService for GrpcAdminServices<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    async fn status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusReply>, Status> {
        let req = request.into_inner();
        return Ok(Response::new(StatusReply::default()));
    }
}
