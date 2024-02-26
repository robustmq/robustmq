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

use super::cluster::Cluster;
use crate::broker_cluster::BrokerCluster;
use crate::client::{create_shard, delete_shard, register_node, unregister_node};
use crate::raft::message::{RaftMessage, RaftResponseMesage};
use crate::storage::cluster_storage::ClusterStorage;
use crate::storage::raft_core::RaftRocksDBStorageCore;
use crate::storage::schema::{StorageData, StorageDataType};
use crate::storage_cluster::StorageCluster;
use bincode::serialize;
use common::errors::RobustMQError;
use common::log::info_meta;
use prost::Message;

use protocol::placement_center::placement::placement_center_service_server::PlacementCenterService;
use protocol::placement_center::placement::{ SendRaftMessageReply, SendRaftMessageRequest,
};
use protocol::placement_center::placement::{
    ClusterType, CommonReply, CreateShardRequest, DeleteShardRequest, GetShardReply,
    GetShardRequest, HeartbeatRequest, RegisterNodeRequest, ReportMonitorRequest,
    SendRaftConfChangeReply, SendRaftConfChangeRequest, UnRegisterNodeRequest,
};
use raft::eraftpb::{ConfChange, Message as raftPreludeMessage};

use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::{self, Receiver};
use tokio::time::timeout;

use std::sync::{Arc, RwLock};
use std::time::{self, Duration, Instant, SystemTime, UNIX_EPOCH};
use tonic::{Request, Response, Status};

pub struct GrpcService {
    cluster: Arc<RwLock<Cluster>>,
    raft_sender: Sender<RaftMessage>,
    rocksdb_storage: Arc<RwLock<RaftRocksDBStorageCore>>,
    cluster_storage: Arc<ClusterStorage>,
    storage_cluster: Arc<RwLock<StorageCluster>>,
    broker_cluster: Arc<RwLock<BrokerCluster>>,
}

impl GrpcService {
    pub fn new(
        cluster: Arc<RwLock<Cluster>>,
        raft_sender: Sender<RaftMessage>,
        rocksdb_storage: Arc<RwLock<RaftRocksDBStorageCore>>,
        cluster_storage: Arc<ClusterStorage>,
        storage_cluster: Arc<RwLock<StorageCluster>>,
        broker_cluster: Arc<RwLock<BrokerCluster>>,
    ) -> Self {
        GrpcService {
            cluster,
            raft_sender,
            rocksdb_storage,
            cluster_storage,
            storage_cluster,
            broker_cluster,
        }
    }

    fn rewrite_leader(&self) -> bool {
        return self.cluster.read().unwrap().is_leader();
    }

    fn verify(&self) -> Result<(), RobustMQError> {
        let cluster = self.cluster.read().unwrap();

        if cluster.leader_alive() {
            return Err(RobustMQError::MetaClusterNotLeaderNode);
        }

        return Ok(());
    }

    async fn apply_raft_machine(&self, data: StorageData, action: String) -> Result<(), RobustMQError> {
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();

        let _ = self
            .raft_sender
            .send(RaftMessage::Propose {
                data: serialize(&data).unwrap(),
                chan: sx,
            })
            .await;

        if !recv_chan_resp(rx).await {
            return Err(RobustMQError::MetaLogCommitTimeout(action));
        }
        return Ok(());
    }
}

async fn recv_chan_resp(rx: Receiver<RaftResponseMesage>) -> bool {
    let res = timeout(Duration::from_secs(30), async {
        match rx.await {
            Ok(val) => {
                return val;
            }
            Err(_) => {
                return RaftResponseMesage::Fail;
            }
        }
    });
    match res.await {
        Ok(_) => return true,
        Err(_) => {
            return false;
        }
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
            let leader_addr = self.cluster.read().unwrap().leader_addr();
            match register_node(&leader_addr, req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::RegisterNode,
            RegisterNodeRequest::encode_to_vec(&req),
        );
        match self
            .apply_raft_machine(data, "register_node".to_string())
            .await
        {
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
            let leader_addr = self.cluster.read().unwrap().leader_addr();
            match unregister_node(&leader_addr, req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::RegisterNode,
            UnRegisterNodeRequest::encode_to_vec(&req),
        );
        match self
            .apply_raft_machine(data, "un_register_node".to_string())
            .await
        {
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
            let leader_addr = self.cluster.read().unwrap().leader_addr();
            match create_shard(&leader_addr, req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::RegisterNode,
            CreateShardRequest::encode_to_vec(&req),
        );
        match self
            .apply_raft_machine(data, "create_shard".to_string())
            .await
        {
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
        let shard_info = self
            .cluster_storage
            .get_shard(req.cluster_name.clone(), req.shard_name);
        let mut result = GetShardReply::default();
        if shard_info.is_none() {
            let si = shard_info.unwrap();
            result.cluster_name = req.cluster_name;
            result.shard_id = si.shard_id;
            result.shard_name = si.shard_name;
            result.replica = si.replica;
            result.replicas = serialize(&si.replicas).unwrap();
        }

        return Ok(Response::new(result));
    }

    async fn delete_shard(
        &self,
        request: Request<DeleteShardRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if self.rewrite_leader() {
            let leader_addr = self.cluster.read().unwrap().leader_addr();
            match delete_shard(&leader_addr, req).await {
                Ok(resp) => return Ok(Response::new(resp)),
                Err(e) => return Err(Status::cancelled(e.to_string())),
            }
        }

        // Params validate

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::RegisterNode,
            DeleteShardRequest::encode_to_vec(&req),
        );
        match self
            .apply_raft_machine(data, "delete_shard".to_string())
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
            let mut sc = self.storage_cluster.write().unwrap();
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

    async fn send_raft_message(
        &self,
        request: Request<SendRaftMessageRequest>,
    ) -> Result<Response<SendRaftMessageReply>, Status> {
        let message = raftPreludeMessage::decode(request.into_inner().message.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();

        match self
            .raft_sender
            .send(RaftMessage::Raft { message, chan: sx })
            .await
        {
            Ok(_) => {
                if !recv_chan_resp(rx).await {
                    return Err(Status::cancelled(
                        RobustMQError::MetaLogCommitTimeout("send_raft_message".to_string())
                            .to_string(),
                    ));
                }
            }
            Err(e) => {
                return Err(Status::aborted(
                    RobustMQError::RaftStepCommitFail(e.to_string()).to_string(),
                ));
            }
        }
        Ok(Response::new(SendRaftMessageReply::default()))
    }

    async fn send_raft_conf_change(
        &self,
        request: Request<SendRaftConfChangeRequest>,
    ) -> Result<Response<SendRaftConfChangeReply>, Status> {
        let change = ConfChange::decode(request.into_inner().message.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let (sx, rx) = oneshot::channel::<RaftResponseMesage>();
        info_meta(&format!(
            "grpc receive send_raft_conf_change change data:{:?}",
            change
        ));
        match self
            .raft_sender
            .send(RaftMessage::ConfChange { change, chan: sx })
            .await
        {
            Ok(_) => {
                if !recv_chan_resp(rx).await {
                    return Err(Status::cancelled(
                        RobustMQError::MetaLogCommitTimeout("send_raft_conf_change".to_string())
                            .to_string(),
                    ));
                }
            }
            Err(e) => {
                return Err(Status::aborted(
                    RobustMQError::RaftStepCommitFail(e.to_string()).to_string(),
                ));
            }
        }
        Ok(Response::new(SendRaftConfChangeReply::default()))
    }
}
