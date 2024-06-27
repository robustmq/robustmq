use crate::core::cache_manager::CacheManager;
use crate::core::error::MQTTBrokerError;
use crate::subscribe::subscribe_cache::SubscribeCacheManager;
use clients::poll::ClientPool;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::errors::RobustMQError;
use metadata_struct::mqtt::cluster;
use protocol::broker_server::generate::mqtt::DeleteSessionRequest;
use protocol::broker_server::generate::mqtt::{
    mqtt_broker_service_server::MqttBrokerService, CommonReply, UpdateCacheRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcBrokerServices {
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeCacheManager>,
    client_poll: Arc<ClientPool>,
}

impl GrpcBrokerServices {
    pub fn new(
        metadata_cache: Arc<CacheManager>,
        subscribe_manager: Arc<SubscribeCacheManager>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        return GrpcBrokerServices {
            cache_manager: metadata_cache,
            subscribe_manager,
            client_poll,
        };
    }
}

#[tonic::async_trait]
impl MqttBrokerService for GrpcBrokerServices {
    async fn update_cache(
        &self,
        request: Request<UpdateCacheRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        return Ok(Response::new(CommonReply::default()));
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if self.cache_manager.cluster_name != req.cluster_name {
            return Err(Status::cancelled("Cluster name does not match".to_string()));
        }

        if req.client_id.is_empty() {
            return Err(Status::cancelled("Client ID cannot be empty".to_string()));
        }
        for client_id in req.client_id {
            self.cache_manager.remove_session(&client_id);
            self.subscribe_manager.remove_client(&client_id);
        }

        return Ok(Response::new(CommonReply::default()));
    }
}
