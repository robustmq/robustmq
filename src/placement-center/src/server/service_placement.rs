/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::cache::cluster::ClusterCache;
use crate::cache::placement::PlacementClusterCache;
use crate::raft::storage::PlacementCenterStorage;
use crate::storage::global_id::GlobalId;
use crate::storage::rocksdb::RocksDBEngine;
use clients::placement_center::placement::{register_node, unregister_node};
use clients::ClientPool;
use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::generate::common::{CommonReply, GenerageIdType};
use protocol::placement_center::generate::placement::placement_center_service_server::PlacementCenterService;
use protocol::placement_center::generate::placement::{
    GenerateUniqueNodeIdReply, GenerateUniqueNodeIdRequest, HeartbeatRequest, RegisterNodeRequest,
    ReportMonitorRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest, SendRaftMessageReply,
    SendRaftMessageRequest, UnRegisterNodeRequest,
};
use raft::eraftpb::{ConfChange, Message as raftPreludeMessage};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub struct GrpcPlacementService {
    placement_center_storage: Arc<PlacementCenterStorage>,
    placement_cache: Arc<RwLock<PlacementClusterCache>>,
    cluster_cache: Arc<RwLock<ClusterCache>>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_poll: Arc<Mutex<ClientPool>>,
}

impl GrpcPlacementService {
    pub fn new(
        placement_center_storage: Arc<PlacementCenterStorage>,
        placement_cache: Arc<RwLock<PlacementClusterCache>>,
        cluster_cache: Arc<RwLock<ClusterCache>>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_poll: Arc<Mutex<ClientPool>>,
    ) -> Self {
        GrpcPlacementService {
            placement_center_storage,
            placement_cache,
            cluster_cache,
            rocksdb_engine_handler,
            client_poll,
        }
    }

    fn rewrite_leader(&self) -> bool {
        return !self.placement_cache.read().unwrap().is_leader();
    }
}

#[tonic::async_trait]
impl PlacementCenterService for GrpcPlacementService {
    async fn register_node(
        &self,
        request: Request<RegisterNodeRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        if self.rewrite_leader() {
            let leader_addr = self.placement_cache.read().unwrap().leader_addr();
            match register_node(self.client_poll.clone(), leader_addr, req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        match self.placement_center_storage.save_node(req).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn un_register_node(
        &self,
        request: Request<UnRegisterNodeRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        if self.rewrite_leader() {
            let leader_addr = self.placement_cache.read().unwrap().leader_addr();
            match unregister_node(self.client_poll.clone(), leader_addr, req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data

        match self.placement_center_storage.delete_node(req).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        // Params validate

        //
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let cluster_type = req.cluster_type();

        let mut sc = self.cluster_cache.write().unwrap();
        sc.heart_time(req.node_id, time);

        return Ok(Response::new(CommonReply::default()));
    }

    async fn report_monitor(
        &self,
        request: Request<ReportMonitorRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        return Ok(Response::new(CommonReply::default()));
    }

    async fn send_raft_message(
        &self,
        request: Request<SendRaftMessageRequest>,
    ) -> Result<Response<SendRaftMessageReply>, Status> {
        let message = raftPreludeMessage::decode(request.into_inner().message.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        match self
            .placement_center_storage
            .save_raft_message(message)
            .await
        {
            Ok(_) => return Ok(Response::new(SendRaftMessageReply::default())),
            Err(e) => {
                return Err(Status::cancelled(
                    RobustMQError::MetaLogCommitTimeout(e.to_string()).to_string(),
                ));
            }
        }
    }

    async fn send_raft_conf_change(
        &self,
        request: Request<SendRaftConfChangeRequest>,
    ) -> Result<Response<SendRaftConfChangeReply>, Status> {
        let change = ConfChange::decode(request.into_inner().message.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        match self
            .placement_center_storage
            .save_conf_raft_message(change)
            .await
        {
            Ok(_) => return Ok(Response::new(SendRaftConfChangeReply::default())),
            Err(e) => {
                return Err(Status::cancelled(
                    RobustMQError::MetaLogCommitTimeout(e.to_string()).to_string(),
                ));
            }
        }
    }

    async fn generate_unique_id(
        &self,
        request: Request<GenerateUniqueNodeIdRequest>,
    ) -> Result<Response<GenerateUniqueNodeIdReply>, Status> {
        let req = request.into_inner();
        let mut resp = GenerateUniqueNodeIdReply::default();
        let generate = GlobalId::new(self.rocksdb_engine_handler.clone());

        if req.generage_type() == GenerageIdType::UniqStr {
            resp.id_str = generate.generate_uniq_str();
            return Ok(Response::new(resp));
        }

        if req.generage_type() == GenerageIdType::UniqInt {
            match generate.generate_uniq_id().await {
                Ok(da) => {
                    resp.id_int = da;
                    return Ok(Response::new(resp));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }
        return Ok(Response::new(resp));
    }
}
