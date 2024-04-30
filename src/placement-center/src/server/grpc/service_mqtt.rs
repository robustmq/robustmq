use std::sync::Arc;

use crate::{
    cache::cluster::ClusterCache, core::lock::Lock, raft::storage::PlacementCenterStorage,
    storage::rocksdb::RocksDBEngine,
};
use protocol::placement_center::generate::mqtt::{
    mqtt_service_server::MqttService, ShareSubReply, ShareSubRequest,
};
use tonic::{Request, Response, Status};

pub struct GrpcMqttService {
    cluster_cache: Arc<ClusterCache>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcMqttService {
    pub fn new(
        cluster_cache: Arc<ClusterCache>,
        placement_center_storage: Arc<PlacementCenterStorage>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcMqttService {
            cluster_cache,
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
        let key = format!("global_share_sub_lock");
        let lock = Lock::new(key.clone(), self.rocksdb_engine_handler.clone());
        lock.lock();

        lock.un_lock();
        return Ok(Response::new(ShareSubReply::default()));
    }
}
