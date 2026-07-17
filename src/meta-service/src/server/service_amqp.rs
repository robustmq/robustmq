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

use crate::core::cache::MetaCacheManager;
use crate::core::notify::{
    send_notify_by_delete_binding, send_notify_by_delete_exchange, send_notify_by_delete_queue,
    send_notify_by_set_binding, send_notify_by_set_exchange, send_notify_by_set_queue,
};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use bytes::Bytes;
use metadata_struct::amqp::binding::AmqpBinding;
use metadata_struct::amqp::exchange::AmqpExchange;
use metadata_struct::amqp::queue::AmqpQueue;
use node_call::NodeCallManager;
use prost::Message;
use prost_validate::Validator;
use protocol::meta::meta_service_amqp::amqp_service_server::AmqpService;
use protocol::meta::meta_service_amqp::{
    DeleteBindingReply, DeleteBindingRequest, DeleteExchangeReply, DeleteExchangeRequest,
    DeleteQueueReply, DeleteQueueRequest, ListBindingReply, ListBindingRequest, ListExchangeReply,
    ListExchangeRequest, ListQueueReply, ListQueueRequest, SetBindingReply, SetBindingRequest,
    SetExchangeReply, SetExchangeRequest, SetQueueReply, SetQueueRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcAmqpService {
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<MetaCacheManager>,
    call_manager: Arc<NodeCallManager>,
}

impl GrpcAmqpService {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        cache_manager: Arc<MetaCacheManager>,
        call_manager: Arc<NodeCallManager>,
    ) -> Self {
        GrpcAmqpService {
            raft_manager,
            cache_manager,
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

        let existing = self
            .cache_manager
            .get_exchange(&req.tenant, &req.exchange_name);

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

        let exchanges = self.cache_manager.list_exchange_by_tenant(&req.tenant);
        let mut encoded = Vec::with_capacity(exchanges.len());
        for exchange in exchanges {
            encoded.push(exchange.encode().map_err(Self::to_status)?);
        }

        Ok(Response::new(ListExchangeReply { exchanges: encoded }))
    }

    async fn set_queue(
        &self,
        request: Request<SetQueueRequest>,
    ) -> Result<Response<SetQueueReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;
        let queue = AmqpQueue::decode(&req.queue).map_err(Self::to_status)?;

        let data = StorageData::new(
            StorageDataType::AmqpSetQueue,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        send_notify_by_set_queue(&self.call_manager, queue)
            .await
            .map_err(Self::to_status)?;

        Ok(Response::new(SetQueueReply {}))
    }

    async fn delete_queue(
        &self,
        request: Request<DeleteQueueRequest>,
    ) -> Result<Response<DeleteQueueReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;

        let existing = self.cache_manager.get_queue(&req.tenant, &req.queue_name);

        let data = StorageData::new(
            StorageDataType::AmqpDeleteQueue,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        if let Some(queue) = existing {
            send_notify_by_delete_queue(&self.call_manager, queue)
                .await
                .map_err(Self::to_status)?;
        }

        Ok(Response::new(DeleteQueueReply {}))
    }

    async fn list_queue(
        &self,
        request: Request<ListQueueRequest>,
    ) -> Result<Response<ListQueueReply>, Status> {
        let req = request.into_inner();

        let queues = self.cache_manager.list_queue_by_tenant(&req.tenant);
        let mut encoded = Vec::with_capacity(queues.len());
        for queue in queues {
            encoded.push(queue.encode().map_err(Self::to_status)?);
        }

        Ok(Response::new(ListQueueReply { queues: encoded }))
    }

    async fn set_binding(
        &self,
        request: Request<SetBindingRequest>,
    ) -> Result<Response<SetBindingReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;
        let binding = AmqpBinding::decode(&req.binding).map_err(Self::to_status)?;

        let data = StorageData::new(
            StorageDataType::AmqpSetBinding,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        send_notify_by_set_binding(&self.call_manager, binding)
            .await
            .map_err(Self::to_status)?;

        Ok(Response::new(SetBindingReply {}))
    }

    async fn delete_binding(
        &self,
        request: Request<DeleteBindingRequest>,
    ) -> Result<Response<DeleteBindingReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;

        let key = format!(
            "{}/{}/{}/{}",
            req.source, req.destination_type, req.destination, req.routing_key
        );
        let existing = self.cache_manager.get_binding(&req.tenant, &key);

        let data = StorageData::new(
            StorageDataType::AmqpDeleteBinding,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        if let Some(binding) = existing {
            send_notify_by_delete_binding(&self.call_manager, binding)
                .await
                .map_err(Self::to_status)?;
        }

        Ok(Response::new(DeleteBindingReply {}))
    }

    async fn list_binding(
        &self,
        request: Request<ListBindingRequest>,
    ) -> Result<Response<ListBindingReply>, Status> {
        let req = request.into_inner();

        let bindings = self.cache_manager.list_binding_by_tenant(&req.tenant);
        let mut encoded = Vec::with_capacity(bindings.len());
        for binding in bindings {
            encoded.push(binding.encode().map_err(Self::to_status)?);
        }

        Ok(Response::new(ListBindingReply { bindings: encoded }))
    }
}
