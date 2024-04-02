use std::sync::{Arc, RwLock};

use common_base::errors::RobustMQError;
use protocol::broker_server::generate::mqtt::{
    mqtt_broker_service_server::MqttBrokerService, UpdateCacheReply, UpdateCacheRequest,
};
use tonic::{Request, Response, Status};

use crate::metadata::cache::MetadataCache;

pub struct GrpcBrokerServices {
    metadata_cache: Arc<RwLock<MetadataCache>>,
}

impl GrpcBrokerServices {
    pub fn new(metadata_cache: Arc<RwLock<MetadataCache>>) -> Self {
        return GrpcBrokerServices { metadata_cache };
    }
}

#[tonic::async_trait]

impl MqttBrokerService for GrpcBrokerServices {
    async fn update_cache(
        &self,
        request: Request<UpdateCacheRequest>,
    ) -> Result<Response<UpdateCacheReply>, Status> {
        let req = request.into_inner();
        if req.data.is_empty() {
            return Err(Status::cancelled(
                RobustMQError::ParameterCannotBeNull("data".to_string()).to_string(),
            ));
        }
        let mut handler = self.metadata_cache.write().unwrap();
        handler.apply(req.data);
        return Ok(Response::new(UpdateCacheReply::default()));
    }
}
