use std::sync::Arc;

use crate::{
    cache::{cluster::ClusterCache, mqtt::MqttCache},
    core::lock::Lock,
    raft::storage::PlacementCenterStorage,
    storage::rocksdb::RocksDBEngine,
};
use protocol::placement_center::generate::mqtt::{
    mqtt_service_server::MqttService, ShareSubReply, ShareSubRequest,
};
use tonic::{Request, Response, Status};

pub struct GrpcMqttService {
    cluster_cache: Arc<ClusterCache>,
    mqtt_cache: Arc<MqttCache>,
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
            rocksdb_engine_handler,
        }
    }
}

#[tonic::async_trait]
impl MqttService for GrpcMqttService {
    async fn share_sub(
        &self,
        request: Request<ShareSubRequest>,
    ) -> Result<Response<ShareSubReply>, Status> {
        let req = request.into_inner();

        let mut reply = ShareSubReply::default();
        if let Some(share_sub) = self
            .mqtt_cache
            .get_share_sub(req.cluster_name.clone(), req.share_sub_name)
        {
            let leader_id = share_sub.leader_broker;
            if let Some(node) = self.cluster_cache.get_node(req.cluster_name, leader_id) {
                reply.broker_id = leader_id;
                reply.broker_ip = node.node_ip;
                reply.extend_info = node.extend;
            }
        }
        let key = format!("global_share_sub_lock");
        let lock = Lock::new(key.clone(), self.rocksdb_engine_handler.clone());
        lock.lock();

        lock.un_lock();
        return Ok(Response::new(reply));
    }
}
