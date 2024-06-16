use crate::{
    cache::{cluster::ClusterCache, mqtt::MqttCache},
    core::share_sub::calc_share_sub_leader,
    raft::apply::{RaftMachineApply, StorageData, StorageDataType},
    storage::{mqtt::user::MQTTUserStorage, rocksdb::RocksDBEngine},
};
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{
        mqtt_service_server::MqttService, CreateUserRequest, DeleteUserRequest,
        GetShareSubLeaderReply, GetShareSubLeaderRequest, ListUserReply, ListUserRequest,
    },
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcMqttService {
    cluster_cache: Arc<ClusterCache>,
    mqtt_cache: Arc<MqttCache>,
    placement_center_storage: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcMqttService {
    pub fn new(
        cluster_cache: Arc<ClusterCache>,
        mqtt_cache: Arc<MqttCache>,
        placement_center_storage: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcMqttService {
            cluster_cache,
            mqtt_cache,
            placement_center_storage,
            rocksdb_engine_handler,
        }
    }
}

impl GrpcMqttService {}

#[tonic::async_trait]
impl MqttService for GrpcMqttService {
    async fn get_share_sub_leader(
        &self,
        request: Request<GetShareSubLeaderRequest>,
    ) -> Result<Response<GetShareSubLeaderReply>, Status> {
        let req = request.into_inner();
        let cluster_name = req.cluster_name;
        let group_name = req.group_name;
        let mut reply = GetShareSubLeaderReply::default();

        let leader_broker = match calc_share_sub_leader(
            cluster_name.clone(),
            group_name.clone(),
            self.cluster_cache.clone(),
        ) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        if let Some(node) = self.cluster_cache.get_node(cluster_name, leader_broker) {
            reply.broker_id = leader_broker;
            reply.broker_addr = node.node_inner_addr;
            reply.extend_info = node.extend;
        }
        return Ok(Response::new(reply));
    }

    async fn list_user(
        &self,
        request: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        let req = request.into_inner();
        let storage = MQTTUserStorage::new(self.rocksdb_engine_handler.clone());
        match storage.list(req.cluster_name, Some(req.username)) {
            Ok(data) => {
                return Ok(Response::new(ListUserReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        let data = StorageData::new(
            StorageDataType::MQTTCreateUser,
            CreateUserRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "create_user".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        let data = StorageData::new(
            StorageDataType::MQTTDeleteUser,
            DeleteUserRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "delete_user".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
