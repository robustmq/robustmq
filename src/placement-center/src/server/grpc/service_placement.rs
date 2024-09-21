// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
use crate::cache::placement::PlacementCacheManager;
use crate::raft::apply::{RaftMachineApply, StorageData, StorageDataType};
use crate::raft::metadata::RaftGroupMetadata;
use crate::storage::placement::config::ResourceConfigStorage;
use crate::storage::placement::idempotent::IdempotentStorage;
use crate::storage::rocksdb::RocksDBEngine;
use clients::placement::placement::call::{heartbeat, register_node, un_register_node};
use clients::poll::ClientPool;
use common_base::error::placement_center::PlacementCenterError;
use prost::Message;
use protocol::placement_center::generate::common::CommonReply;
use protocol::placement_center::generate::placement::placement_center_service_server::PlacementCenterService;
use protocol::placement_center::generate::placement::{
    ClusterStatusReply, ClusterStatusRequest, DeleteIdempotentDataRequest,
    DeleteResourceConfigRequest, ExistsIdempotentDataReply, ExistsIdempotentDataRequest,
    GetResourceConfigReply, GetResourceConfigRequest, HeartbeatRequest, NodeListReply,
    NodeListRequest, RegisterNodeRequest, ReportMonitorRequest, SendRaftConfChangeReply,
    SendRaftConfChangeRequest, SendRaftMessageReply, SendRaftMessageRequest,
    SetIdempotentDataRequest, SetResourceConfigRequest, UnRegisterNodeRequest,
};
use raft::eraftpb::{ConfChange, Message as raftPreludeMessage};
use std::sync::{Arc, RwLock};
use tonic::{Request, Response, Status};

pub struct GrpcPlacementService {
    placement_center_storage: Arc<RaftMachineApply>,
    raft_metadata: Arc<RwLock<RaftGroupMetadata>>,
    cluster_cache: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_poll: Arc<ClientPool>,
}

impl GrpcPlacementService {
    pub fn new(
        raft_machine_apply: Arc<RaftMachineApply>,
        raft_metadata: Arc<RwLock<RaftGroupMetadata>>,
        cluster_cache: Arc<PlacementCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        GrpcPlacementService {
            placement_center_storage: raft_machine_apply,
            raft_metadata,
            cluster_cache,
            rocksdb_engine_handler,
            client_poll,
        }
    }

    fn rewrite_leader(&self) -> bool {
        return !self.raft_metadata.read().unwrap().is_leader();
    }
}

#[tonic::async_trait]
impl PlacementCenterService for GrpcPlacementService {
    async fn cluster_status(
        &self,
        _: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusReply>, Status> {
        let mut reply = ClusterStatusReply::default();
        match self.raft_metadata.read() {
            Ok(data) => {
                if let Some(leader) = data.leader.clone() {
                    reply.leader = format!("{}@{}", leader.node_ip, leader.node_id);
                }

                let mut nodes = Vec::new();
                for (_, node) in data.peers.clone() {
                    nodes.push(format!("{}@{}", node.node_ip, node.node_id));
                }

                reply.nodes = nodes;
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }

        return Ok(Response::new(reply));
    }

    async fn node_list(
        &self,
        request: Request<NodeListRequest>,
    ) -> Result<Response<NodeListReply>, Status> {
        let req = request.into_inner();
        let mut nodes = Vec::new();
        if let Some(node_list) = self.cluster_cache.node_list.get(&req.cluster_name) {
            for (_, node) in node_list.clone() {
                nodes.push(node.encode())
            }
        }
        return Ok(Response::new(NodeListReply { nodes }));
    }

    async fn register_node(
        &self,
        request: Request<RegisterNodeRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        if self.rewrite_leader() {
            let leader_addr = self.raft_metadata.read().unwrap().leader_addr();
            match register_node(self.client_poll.clone(), vec![leader_addr], req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        let data = StorageData::new(
            StorageDataType::ClusterRegisterNode,
            RegisterNodeRequest::encode_to_vec(&req),
        );
        match self
            .placement_center_storage
            .apply_propose_message(data, "register_node".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::internal(e.to_string()));
            }
        }
    }

    async fn un_register_node(
        &self,
        request: Request<UnRegisterNodeRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        if self.rewrite_leader() {
            let leader_addr = self.raft_metadata.read().unwrap().leader_addr();
            match un_register_node(self.client_poll.clone(), vec![leader_addr], req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        let data = StorageData::new(
            StorageDataType::ClusterUngisterNode,
            UnRegisterNodeRequest::encode_to_vec(&req),
        );
        match self
            .placement_center_storage
            .apply_propose_message(data, "un_register_node".to_string())
            .await
        {
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

        if self.rewrite_leader() {
            let leader_addr = self.raft_metadata.read().unwrap().leader_addr();
            match heartbeat(self.client_poll.clone(), vec![leader_addr], req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        let data = StorageData::new(
            StorageDataType::ClusterNodeHeartbeat,
            HeartbeatRequest::encode_to_vec(&req),
        );
        match self
            .placement_center_storage
            .apply_propose_message(data, "node_hearbeat".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn report_monitor(
        &self,
        _: Request<ReportMonitorRequest>,
    ) -> Result<Response<CommonReply>, Status> {
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
            .apply_raft_message(message, "send_raft_message".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(SendRaftMessageReply::default())),
            Err(e) => {
                return Err(Status::cancelled(
                    PlacementCenterError::RaftLogCommitTimeout(e.to_string()).to_string(),
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
            .apply_conf_raft_message(change, "send_conf_raft_message".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(SendRaftConfChangeReply::default())),
            Err(e) => {
                return Err(Status::cancelled(
                    PlacementCenterError::RaftLogCommitTimeout(e.to_string()).to_string(),
                ));
            }
        }
    }

    async fn set_resource_config(
        &self,
        request: Request<SetResourceConfigRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::ClusterSetResourceConfig,
            SetResourceConfigRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "set_resource_config".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn get_resource_config(
        &self,
        request: Request<GetResourceConfigRequest>,
    ) -> Result<Response<GetResourceConfigReply>, Status> {
        let req = request.into_inner();
        let storage = ResourceConfigStorage::new(self.rocksdb_engine_handler.clone());
        match storage.get(req.cluster_name, req.resources) {
            Ok(data) => {
                if let Some(res) = data {
                    return Ok(Response::new(GetResourceConfigReply { config: res }));
                } else {
                    return Ok(Response::new(GetResourceConfigReply { config: Vec::new() }));
                }
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_resource_config(
        &self,
        request: Request<DeleteResourceConfigRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::ClusterDeleteResourceConfig,
            DeleteResourceConfigRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "delete_resource_config".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn set_idempotent_data(
        &self,
        request: Request<SetIdempotentDataRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::ClusterSetIdempotentData,
            SetIdempotentDataRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "set_idempotent_data".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn exists_idempotent_data(
        &self,
        request: Request<ExistsIdempotentDataRequest>,
    ) -> Result<Response<ExistsIdempotentDataReply>, Status> {
        let req = request.into_inner();
        let storage = IdempotentStorage::new(self.rocksdb_engine_handler.clone());
        match storage.exists(&req.cluster_name, &req.producer_id, req.seq_num) {
            Ok(flag) => {
                return Ok(Response::new(ExistsIdempotentDataReply { exists: flag }));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_idempotent_data(
        &self,
        request: Request<DeleteIdempotentDataRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::ClusterDeleteIdempotentData,
            DeleteIdempotentDataRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "delete_idempotent_data".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
