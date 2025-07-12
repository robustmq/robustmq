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

use crate::admin::inner::{
    delete_session_by_req, send_last_will_message_by_req, update_cache_by_req,
};
use crate::bridge::manager::ConnectorManager;
use crate::handler::cache::CacheManager;
use crate::subscribe::manager::SubscribeManager;
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_inner::mqtt_broker_inner_service_server::MqttBrokerInnerService;
use protocol::broker_mqtt::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateMqttCacheReply, UpdateMqttCacheRequest,
};
use schema_register::schema::SchemaRegisterManager;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tonic::{Request, Response, Status};

pub struct GrpcInnerServices {
    cache_manager: Arc<CacheManager>,
    connector_manager: Arc<ConnectorManager>,
    subscribe_manager: Arc<SubscribeManager>,
    schema_manager: Arc<SchemaRegisterManager>,
    client_pool: Arc<ClientPool>,
    message_storage_adapter: ArcStorageAdapter,
}

impl GrpcInnerServices {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        subscribe_manager: Arc<SubscribeManager>,
        connector_manager: Arc<ConnectorManager>,
        schema_manager: Arc<SchemaRegisterManager>,
        client_pool: Arc<ClientPool>,
        message_storage_adapter: ArcStorageAdapter,
    ) -> Self {
        GrpcInnerServices {
            cache_manager,
            subscribe_manager,
            connector_manager,
            client_pool,
            message_storage_adapter,
            schema_manager,
        }
    }
}

#[tonic::async_trait]
impl MqttBrokerInnerService for GrpcInnerServices {
    async fn update_cache(
        &self,
        request: Request<UpdateMqttCacheRequest>,
    ) -> Result<Response<UpdateMqttCacheReply>, Status> {
        let req = request.into_inner();
        update_cache_by_req(
            &self.cache_manager,
            &self.connector_manager,
            &self.subscribe_manager,
            &self.schema_manager,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

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
            &self.message_storage_adapter,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }
}
