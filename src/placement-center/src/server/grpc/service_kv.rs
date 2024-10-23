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

use std::sync::Arc;

use common_base::error::common::CommonError;
use prost::Message;
use protocol::placement_center::placement_center_kv::kv_service_server::KvService;
use protocol::placement_center::placement_center_kv::{
    DeleteReply, DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetReply,
    SetRequest,
};
use tonic::{Request, Response, Status};

use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::storage::placement::kv::KvStorage;
use crate::storage::rocksdb::RocksDBEngine;

pub struct GrpcKvService {
    raft_machine_apply: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcKvService {
    pub fn new(
        raft_machine_apply: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcKvService {
            raft_machine_apply,
            rocksdb_engine_handler,
        }
    }
}

#[tonic::async_trait]
impl KvService for GrpcKvService {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetReply>, Status> {
        let req = request.into_inner();

        if req.key.is_empty() || req.value.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("key or value".to_string()).to_string(),
            ));
        }

        // Raft state machine is used to store Node data
        let data = StorageData::new(StorageDataType::KvSet, SetRequest::encode_to_vec(&req));
        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(SetReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetReply>, Status> {
        let req = request.into_inner();

        if req.key.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("key".to_string()).to_string(),
            ));
        }

        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        let mut reply = GetReply::default();
        match kv_storage.get(req.key) {
            Ok(Some(data)) => {
                reply.value = data;
                return Ok(Response::new(reply));
            }
            Ok(None) => {}
            Err(e) => return Err(Status::cancelled(e.to_string())),
        }

        return Ok(Response::new(reply));
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteReply>, Status> {
        let req = request.into_inner();

        if req.key.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("key".to_string()).to_string(),
            ));
        }

        // Raft state machine is used to store Node data
        let data = StorageData::new(
            StorageDataType::KvDelete,
            DeleteRequest::encode_to_vec(&req),
        );
        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(DeleteReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn exists(
        &self,
        request: Request<ExistsRequest>,
    ) -> Result<Response<ExistsReply>, Status> {
        let req = request.into_inner();

        if req.key.is_empty() {
            return Err(Status::cancelled(
                CommonError::ParameterCannotBeNull("key".to_string()).to_string(),
            ));
        }

        let kv_storage = KvStorage::new(self.rocksdb_engine_handler.clone());
        match kv_storage.exists(req.key) {
            Ok(flag) => {
                return Ok(Response::new(ExistsReply { flag }));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
