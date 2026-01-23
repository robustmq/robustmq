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

use crate::core::cache::MQTTCacheManager;
use crate::core::inner::{delete_session_by_req, send_last_will_message_by_req};
use crate::subscribe::manager::SubscribeManager;
use grpc_clients::pool::ClientPool;
use protocol::broker::broker_mqtt::broker_mqtt_service_server::BrokerMqttService;
use protocol::broker::broker_mqtt::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tonic::{Request, Response, Status};

pub struct GrpcInnerServices {
    cache_manager: Arc<MQTTCacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_pool: Arc<ClientPool>,
    storage_driver_manager: Arc<StorageDriverManager>,
}

impl GrpcInnerServices {
    pub fn new(
        cache_manager: Arc<MQTTCacheManager>,
        subscribe_manager: Arc<SubscribeManager>,
        client_pool: Arc<ClientPool>,
        storage_driver_manager: Arc<StorageDriverManager>,
    ) -> Self {
        GrpcInnerServices {
            cache_manager,
            subscribe_manager,
            client_pool,
            storage_driver_manager,
        }
    }
}

#[tonic::async_trait]
impl BrokerMqttService for GrpcInnerServices {
    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<DeleteSessionReply>, Status> {
        let req = request.into_inner();
        delete_session_by_req(&self.cache_manager, &self.subscribe_manager, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn send_last_will_message(
        &self,
        request: Request<SendLastWillMessageRequest>,
    ) -> Result<Response<SendLastWillMessageReply>, Status> {
        let req = request.into_inner();
        send_last_will_message_by_req(
            &self.cache_manager,
            &self.client_pool,
            &self.storage_driver_manager,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }
}
