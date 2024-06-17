use crate::core::metadata_cache::MetadataCacheManager;
use crate::{
    metadata::available_flag,
    storage::{cluster::ClusterStorage, user::UserStorage},
};
use clients::poll::ClientPool;
use common_base::{errors::RobustMQError, tools::now_mills};
use metadata_struct::mqtt::user::MQTTUser;
use protocol::{
    broker_server::generate::mqtt::{
        mqtt_broker_service_server::MqttBrokerService, CommonReply, CreateUserRequest,
        SetClusterConfigRequest, UpdateCacheRequest,
    },
    mqtt::{qos, QoS},
};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tonic::{Request, Response, Status};

pub struct GrpcBrokerServices<T> {
    metadata_cache: Arc<MetadataCacheManager>,
    metadata_storage_adapter: Arc<T>,
    client_poll: Arc<ClientPool>,
}

impl<T> GrpcBrokerServices<T>
where
    T: StorageAdapter,
{
    pub fn new(
        metadata_cache: Arc<MetadataCacheManager>,
        metadata_storage_adapter: Arc<T>,

        client_poll: Arc<ClientPool>,
    ) -> Self {
        return GrpcBrokerServices {
            metadata_cache,
            metadata_storage_adapter,
            client_poll,
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
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if req.data.is_empty() {
            return Err(Status::cancelled(
                RobustMQError::ParameterCannotBeNull("data".to_string()).to_string(),
            ));
        }
        self.metadata_cache.apply(req.data);
        return Ok(Response::new(CommonReply::default()));
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if req.username.is_empty() || req.password.is_empty() {
            return Err(Status::cancelled(
                RobustMQError::ParameterCannotBeNull("username or password".to_string())
                    .to_string(),
            ));
        }
        let user_info = MQTTUser {
            username: req.username.clone(),
            password: req.password,
            is_superuser: true,
        };
        let user_storage = UserStorage::new(self.client_poll.clone());
        match user_storage.save_user(user_info.clone()).await {
            Ok(_) => {
                let mut reply = CommonReply::default();
                reply.data = req.username;
                self.metadata_cache.add_user(user_info);
                return Ok(Response::new(reply));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn set_cluster_config(
        &self,
        request: Request<SetClusterConfigRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let mut cluster = self.metadata_cache.get_cluster_info();
        let req_data = request.into_inner();
        cluster.session_expiry_interval = req_data.session_expiry_interval;
        cluster.topic_alias_max = req_data.topic_alias_max as u16;
        cluster.max_qos = if let Some(data) = qos(req_data.max_qos as u8) {
            data
        } else {
            QoS::AtLeastOnce
        };

        cluster.retain_available = available_flag(req_data.retain_available.try_into().unwrap());
        cluster.wildcard_subscription_available =
            available_flag(req_data.wildcard_subscription_available.try_into().unwrap());
        cluster.subscription_identifiers_available = available_flag(
            req_data
                .subscription_identifiers_available
                .try_into()
                .unwrap(),
        );
        cluster.shared_subscription_available =
            available_flag(req_data.shared_subscription_available.try_into().unwrap());
        cluster.max_packet_size = req_data.max_packet_size;
        cluster.server_keep_alive = req_data.server_keep_alive as u16;
        cluster.receive_max = req_data.receive_max as u16;
        cluster.secret_free_login = req_data.secret_free_login;

        self.metadata_cache.set_cluster_info(cluster.clone());

        let cluster_storage = ClusterStorage::new(self.metadata_storage_adapter.clone());
        match cluster_storage.save_cluster(cluster.clone()).await {
            Ok(_) => {
                return Ok(Response::new(CommonReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use protocol::broker_server::generate::mqtt::{
        mqtt_broker_service_client::MqttBrokerServiceClient, Available, CreateUserRequest,
        CreateUserSalt, SetClusterConfigRequest,
    };

    #[tokio::test]
    async fn create_user() {
        let mut client = MqttBrokerServiceClient::connect("http://127.0.0.1:9981")
            .await
            .unwrap();

        let request = CreateUserRequest {
            username: "lobo".to_string(),
            password: "lobo@123".to_string(),
            is_super_user: true,
            salt: CreateUserSalt::Default.into(),
        };
        let response = client
            .create_user(tonic::Request::new(request))
            .await
            .unwrap();

        println!("response={:?}", response);
    }

    #[tokio::test]
    async fn set_cluster_config() {
        let mut client = MqttBrokerServiceClient::connect("http://127.0.0.1:9981")
            .await
            .unwrap();

        let request = SetClusterConfigRequest {
            session_expiry_interval: 1800000,
            topic_alias_max: 256,
            max_qos: 2,
            retain_available: Available::Disable.into(),
            wildcard_subscription_available: Available::Disable.into(),
            max_packet_size: 1024 * 1024,
            subscription_identifiers_available: Available::Disable.into(),
            shared_subscription_available: Available::Disable.into(),
            receive_max: 1024,
            secret_free_login: false,
            server_keep_alive: 10000,
        };
        let response = client
            .set_cluster_config(tonic::Request::new(request))
            .await
            .unwrap();

        println!("response={:?}", response);
    }
}
