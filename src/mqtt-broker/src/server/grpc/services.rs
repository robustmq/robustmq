use std::sync::Arc;

use common_base::errors::RobustMQError;
use protocol::broker_server::generate::mqtt::{
    mqtt_broker_service_server::MqttBrokerService, CreateUserReply, CreateUserRequest,
    UpdateCacheReply, UpdateCacheRequest,
};
use storage_adapter::adapter::memory::MemoryStorageAdapter;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::{
    metadata::{cache::MetadataCache, user::User},
    storage::user::UserStorage,
};

pub struct GrpcBrokerServices {
    metadata_cache: Arc<RwLock<MetadataCache>>,
    storage_adapter: Arc<MemoryStorageAdapter>,
}

impl GrpcBrokerServices {
    pub fn new(
        metadata_cache: Arc<RwLock<MetadataCache>>,
        storage_adapter: Arc<MemoryStorageAdapter>,
    ) -> Self {
        return GrpcBrokerServices {
            metadata_cache,
            storage_adapter,
        };
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
        let mut handler = self.metadata_cache.write().await;
        handler.apply(req.data);
        return Ok(Response::new(UpdateCacheReply::default()));
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        let req = request.into_inner();
        if req.username.is_empty() || req.password.is_empty() {
            return Err(Status::cancelled(
                RobustMQError::ParameterCannotBeNull("username or password".to_string())
                    .to_string(),
            ));
        }
        let user_info = User {
            username: req.username,
            password: req.password,
        };
        let user_storage = UserStorage::new(self.storage_adapter.clone());
        match user_storage.save_user(user_info).await {
            Ok(_) => {
                return Ok(Response::new(CreateUserReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
