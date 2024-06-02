use std::sync::Arc;

use crate::{
    cache::{cluster::ClusterCache, mqtt::MqttCache},
    core::{lock::Lock, share_sub::calc_share_sub_leader},
    raft::storage::PlacementCenterStorage,
    storage::rocksdb::RocksDBEngine,
    structs::share_sub::ShareSub,
};
use common_base::{errors::RobustMQError, log::info, tools::now_second};
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{
        mqtt_service_server::MqttService, DeleteShareSubLeaderRequest, GetShareSubLeaderReply,
        GetShareSubLeaderRequest,
    },
};
use tonic::{Request, Response, Status};

pub struct GrpcMqttService {
    cluster_cache: Arc<ClusterCache>,
    mqtt_cache: Arc<MqttCache>,
    placement_center_storage: Arc<PlacementCenterStorage>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcMqttService {
    pub fn new(
        cluster_cache: Arc<ClusterCache>,
        mqtt_cache: Arc<MqttCache>,
        placement_center_storage: Arc<PlacementCenterStorage>,
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

    async fn delete_share_sub_leader(
        &self,
        request: Request<DeleteShareSubLeaderRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let cluster_name = req.cluster_name.clone();
        let group_name = req.group_name.clone();

        let reply = CommonReply::default();

        if let Some(_) = self
            .mqtt_cache
            .get_share_sub(cluster_name.clone(), group_name.clone())
        {
            self.mqtt_cache
                .remove_share_sub(cluster_name.clone(), group_name.clone());

            match self.placement_center_storage.delete_share_sub(req).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }

            return Ok(Response::new(reply));
        }
        return Err(Status::cancelled(
            RobustMQError::ResourceDoesNotExist.to_string(),
        ));
    }
}
