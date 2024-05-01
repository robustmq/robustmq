use std::sync::Arc;

use crate::{
    cache::{cluster::ClusterCache, mqtt::MqttCache},
    core::{lock::Lock, share_sub::calc_share_sub_leader},
    raft::storage::PlacementCenterStorage,
    storage::rocksdb::RocksDBEngine,
    structs::share_sub::ShareSub,
};
use common_base::{errors::RobustMQError, tools::now_second};
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{
        mqtt_service_server::MqttService, DeleteShareSubRequest, GetShareSubReply,
        GetShareSubRequest,
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

#[tonic::async_trait]
impl MqttService for GrpcMqttService {
    async fn get_share_sub(
        &self,
        request: Request<GetShareSubRequest>,
    ) -> Result<Response<GetShareSubReply>, Status> {
        let req = request.into_inner();
        let cluster_name = req.cluster_name;
        let group_name = req.group_name;
        let sub_name = req.sub_name;

        let mut reply = GetShareSubReply::default();
        let leader_broker = if let Some(share_sub) = self
            .mqtt_cache
            .get_share_sub(cluster_name.clone(), sub_name.clone())
        {
            share_sub.leader_broker
        } else {
            let key = format!("global_share_sub_lock");
            let lock = Lock::new(key.clone(), self.rocksdb_engine_handler.clone());
            lock.lock().unwrap();
            let leader_broker =
                match calc_share_sub_leader(cluster_name.clone(), self.cluster_cache.clone()) {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(Status::cancelled(e.to_string()));
                    }
                };

            let share_sub = ShareSub {
                cluster_name: cluster_name.clone(),
                group_name: group_name.clone(),
                sub_name: sub_name.clone(),
                leader_broker,
                create_time: now_second(),
            };

            self.mqtt_cache.add_share_sub(
                cluster_name.clone(),
                group_name.clone(),
                share_sub.clone(),
            );

            match self
                .placement_center_storage
                .save_share_sub(share_sub)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }

            lock.un_lock().unwrap();
            leader_broker
        };
        if let Some(node) = self.cluster_cache.get_node(cluster_name, leader_broker) {
            reply.broker_id = leader_broker;
            reply.broker_ip = node.node_ip;
            reply.extend_info = node.extend;
        }
        return Ok(Response::new(reply));
    }

    async fn delete_share_sub(
        &self,
        request: Request<DeleteShareSubRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let cluster_name = req.cluster_name.clone();
        let group_name = req.group_name.clone();
        let sub_name = req.sub_name.clone();

        let reply = CommonReply::default();

        if let Some(_) = self
            .mqtt_cache
            .get_share_sub(cluster_name.clone(), sub_name.clone())
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
