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
use crate::raft::storage::PlacementCenterStorage;
use protocol::placement_center::generate::{
    common::CommonReply,
    kv::{
        kv_service_server::KvService, DeleteRequest, ExistsReply, ExistsRequest, GetReply,
        GetRequest, SetRequest,
    },
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcKvService {
    placement_center_storage: Arc<PlacementCenterStorage>,
}

impl GrpcKvService {
    pub fn new(placement_center_storage: Arc<PlacementCenterStorage>) -> Self {
        GrpcKvService {
            placement_center_storage,
        }
    }
}

#[tonic::async_trait]
impl KvService for GrpcKvService {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        // Params validate

        // Raft state machine is used to store Node data
        match self.placement_center_storage.set(req).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetReply>, Status> {
        let resp = GetReply::default();
        return Ok(Response::new(resp));
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        // Params validate

        // Raft state machine is used to store Node data
        match self.placement_center_storage.delete(req).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn exists(
        &self,
        request: Request<ExistsRequest>,
    ) -> Result<Response<ExistsReply>, Status> {
        let resp = ExistsReply::default();
        return Ok(Response::new(resp));
    }
}
