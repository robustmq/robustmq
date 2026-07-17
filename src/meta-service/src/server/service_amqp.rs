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

use crate::core::notify::{send_notify_by_delete_exchange, send_notify_by_set_exchange};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::amqp::exchange::AmqpExchangeStorage;
use bytes::Bytes;
use metadata_struct::amqp::exchange::AmqpExchange;
use node_call::NodeCallManager;
use prost::Message;
use prost_validate::Validator;
use protocol::meta::meta_service_amqp::amqp_service_server::AmqpService;
use protocol::meta::meta_service_amqp::{
    DeleteExchangeReply, DeleteExchangeRequest, ListExchangeReply, ListExchangeRequest,
    SetExchangeReply, SetExchangeRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcAmqpService {
    raft_manager: Arc<MultiRaftManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    call_manager: Arc<NodeCallManager>,
}

impl GrpcAmqpService {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        call_manager: Arc<NodeCallManager>,
    ) -> Self {
        GrpcAmqpService {
            raft_manager,
            rocksdb_engine_handler,
            call_manager,
        }
    }

    fn validate_request<T: Validator>(req: &T) -> Result<(), Status> {
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))
    }

    fn to_status<E: ToString>(e: E) -> Status {
        Status::internal(e.to_string())
    }
}

#[tonic::async_trait]
impl AmqpService for GrpcAmqpService {
    async fn set_exchange(
        &self,
        request: Request<SetExchangeRequest>,
    ) -> Result<Response<SetExchangeReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;
        let exchange = AmqpExchange::decode(&req.exchange).map_err(Self::to_status)?;

        let data = StorageData::new(
            StorageDataType::AmqpSetExchange,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        send_notify_by_set_exchange(&self.call_manager, exchange)
            .await
            .map_err(Self::to_status)?;

        Ok(Response::new(SetExchangeReply {}))
    }

    async fn delete_exchange(
        &self,
        request: Request<DeleteExchangeRequest>,
    ) -> Result<Response<DeleteExchangeReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;

        let storage = AmqpExchangeStorage::new(self.rocksdb_engine_handler.clone());
        let existing = storage
            .get(&req.tenant, &req.exchange_name)
            .map_err(Self::to_status)?;

        let data = StorageData::new(
            StorageDataType::AmqpDeleteExchange,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        if let Some(exchange) = existing {
            send_notify_by_delete_exchange(&self.call_manager, exchange)
                .await
                .map_err(Self::to_status)?;
        }

        Ok(Response::new(DeleteExchangeReply {}))
    }

    async fn list_exchange(
        &self,
        request: Request<ListExchangeRequest>,
    ) -> Result<Response<ListExchangeReply>, Status> {
        let req = request.into_inner();

        let storage = AmqpExchangeStorage::new(self.rocksdb_engine_handler.clone());
        let exchanges = storage
            .list_by_tenant(&req.tenant)
            .map_err(Self::to_status)?;
        let mut encoded = Vec::with_capacity(exchanges.len());
        for exchange in exchanges {
            encoded.push(exchange.encode().map_err(Self::to_status)?);
        }

        Ok(Response::new(ListExchangeReply { exchanges: encoded }))
    }
}
