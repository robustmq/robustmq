use crate::{
    metadata::{cache::MetadataCache, user::User},
    storage::user::UserStorage,
};
use common_base::{errors::RobustMQError, tools::now_mills};
use protocol::broker_server::generate::mqtt::{
    mqtt_broker_service_server::MqttBrokerService, CreateUserReply, CreateUserRequest,
    UpdateCacheReply, UpdateCacheRequest,
};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

pub struct GrpcBrokerServices<T> {
    metadata_cache: Arc<RwLock<MetadataCache<T>>>,
    metadata_storage_adapter: Arc<T>,
}

impl<T> GrpcBrokerServices<T>
where
    T: StorageAdapter,
{
    pub fn new(
        metadata_cache: Arc<RwLock<MetadataCache<T>>>,
        metadata_storage_adapter: Arc<T>,
    ) -> Self {
        return GrpcBrokerServices {
            metadata_cache,
            metadata_storage_adapter,
        };
    }
}

#[tonic::async_trait]
impl<T> MqttBrokerService for GrpcBrokerServices<T>
where
    T: StorageAdapter + Send + Sync + 'static,
{
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
            salt: crate::metadata::user::UserSalt::Md5,
            is_superuser: true,
            create_time: now_mills(),
        };
        let user_storage = UserStorage::new(self.metadata_storage_adapter.clone());
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
