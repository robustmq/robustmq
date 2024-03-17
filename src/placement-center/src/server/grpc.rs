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
use crate::cache::engine::EngineClusterCache;
use crate::cache::placement::PlacementClusterCache;
use crate::raft::storage::PlacementCenterStorage;
use clients::placement_center::{
    create_segment, create_shard, delete_segment, delete_shard, register_node, unregister_node,
};
use clients::ClientPool;
use common_base::errors::RobustMQError;
use prost::Message;
use protocol::placement_center::placement::placement_center_service_server::PlacementCenterService;
use protocol::placement_center::placement::{
    ClusterType, CommonReply, CreateSegmentRequest, CreateShardRequest, DeleteSegmentRequest, DeleteShardRequest, GenerateUniqueNodeIdReply, GenerateUniqueNodeIdRequest, GetShardReply, GetShardRequest, HeartbeatRequest, RaftTransferLeaderRequest, RegisterNodeRequest, ReportMonitorRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest, UnRegisterNodeRequest
};
use protocol::placement_center::placement::{SendRaftMessageReply, SendRaftMessageRequest};
use raft::eraftpb::{ConfChange, Message as raftPreludeMessage};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub struct GrpcService {
    placement_center_storage: Arc<PlacementCenterStorage>,
    placement_cache: Arc<RwLock<PlacementClusterCache>>,
    engine_cache: Arc<RwLock<EngineClusterCache>>,
    client_poll: Arc<Mutex<ClientPool>>,
}

impl GrpcService {
    pub fn new(
        placement_center_storage: Arc<PlacementCenterStorage>,
        placement_cache: Arc<RwLock<PlacementClusterCache>>,
        engine_cache: Arc<RwLock<EngineClusterCache>>,
        client_poll: Arc<Mutex<ClientPool>>,
    ) -> Self {
        GrpcService {
            placement_center_storage,
            placement_cache,
            engine_cache,
            client_poll,
        }
    }

    fn rewrite_leader(&self) -> bool {
        return !self.placement_cache.read().unwrap().is_leader();
    }

    fn verify(&self) -> Result<(), RobustMQError> {
        let cluster = self.placement_cache.read().unwrap();

        if cluster.leader_alive() {
            return Err(RobustMQError::MetaClusterNotLeaderNode);
        }

        return Ok(());
    }
}

#[tonic::async_trait]
impl PlacementCenterService for GrpcService {
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

    async fn create_shard(
        &self,
        request: Request<CreateShardRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        if self.rewrite_leader() {
            let leader_addr = self.placement_cache.read().unwrap().leader_addr();
            match create_shard(self.client_poll.clone(), leader_addr, req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        match self.placement_center_storage.save_shard(req).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_shard(
        &self,
        request: Request<DeleteShardRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if self.rewrite_leader() {
            let leader_addr = self.placement_cache.read().unwrap().leader_addr();
            match delete_shard(self.client_poll.clone(), leader_addr, req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        match self.placement_center_storage.delete_shard(req).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn get_shard(
        &self,
        request: Request<GetShardRequest>,
    ) -> Result<Response<GetShardReply>, Status> {
        let req = request.into_inner();
        // let shard_info = self
        //     .cluster_storage
        //     .get_shard(req.cluster_name.clone(), req.shard_name);
        let mut result = GetShardReply::default();
        // if shard_info.is_none() {
        //     let si = shard_info.unwrap();
        //     result.cluster_name = req.cluster_name;
        //     result.shard_id = si.shard_id;
        //     result.shard_name = si.shard_name;
        //     result.replica = si.replica;
        //     result.replicas = serialize(&si.replicas).unwrap();
        // }

        return Ok(Response::new(result));
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

        if cluster_type.eq(&ClusterType::BrokerServer) {
            //todo
        }

        if cluster_type.eq(&ClusterType::StorageEngine) {
            let mut sc = self.engine_cache.write().unwrap();
            sc.heart_time(req.node_id, time);
        }

        return Ok(Response::new(CommonReply::default()));
    }

    async fn report_monitor(
        &self,
        request: Request<ReportMonitorRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        return Ok(Response::new(CommonReply::default()));
    }

    async fn create_segment(
        &self,
        request: Request<CreateSegmentRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if self.rewrite_leader() {
            let leader_addr = self.placement_cache.read().unwrap().leader_addr();
            match create_segment(self.client_poll.clone(), leader_addr, req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        match self.placement_center_storage.create_segment(req).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_segment(
        &self,
        request: Request<DeleteSegmentRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if self.rewrite_leader() {
            let leader_addr = self.placement_cache.read().unwrap().leader_addr();
            match delete_segment(self.client_poll.clone(), leader_addr, req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        match self.placement_center_storage.delete_segment(req).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
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

    async fn raft_transfer_leader(
        &self,
        request: Request<RaftTransferLeaderRequest>,
    ) -> Result<Response<CommonReply>, Status> {

        match self
            .placement_center_storage
            .transfer_leader(request.into_inner().node_id)
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(
                    RobustMQError::MetaLogCommitTimeout(e.to_string()).to_string(),
                ));
            }
        }
    }

    async fn generate_unique_node_id(
        &self,
        request: Request<GenerateUniqueNodeIdRequest>,
    ) -> Result<Response<GenerateUniqueNodeIdReply>, Status> {

        let resp = GenerateUniqueNodeIdReply::default();
        return Ok(Response::new(resp));
    }

}
