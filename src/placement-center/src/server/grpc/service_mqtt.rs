use std::sync::Arc;

use protocol::placement_center::generate::mqtt::{
    mqtt_service_server::MqttService, ShareSubReply, ShareSubRequest,
};
use tonic::{Request, Response, Status};
use crate::{core::lock::Lock, raft::storage::PlacementCenterStorage, storage::rocksdb::RocksDBEngine};


pub struct GrpcMqttService {
    placement_center_storage: Arc<PlacementCenterStorage>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcMqttService {
    pub fn new(
        placement_center_storage: Arc<PlacementCenterStorage>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcMqttService {
            placement_center_storage,
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
