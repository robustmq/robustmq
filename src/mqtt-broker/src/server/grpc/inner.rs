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

use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use log::{debug, info};
use metadata_struct::mqtt::lastwill::LastWillData;
use protocol::broker_mqtt::broker_mqtt_inner::mqtt_broker_inner_service_server::MqttBrokerInnerService;
use protocol::broker_mqtt::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateMqttCacheReply, UpdateMqttCacheRequest,
};
use schema_register::schema::SchemaRegisterManager;
use storage_adapter::storage::StorageAdapter;
use tonic::{Request, Response, Status};

use crate::bridge::manager::ConnectorManager;
use crate::handler::cache::CacheManager;
use crate::handler::cache_update::update_cache_metadata;
use crate::handler::lastwill::send_last_will_message;
use crate::subscribe::subscribe_manager::SubscribeManager;

pub struct GrpcInnerServices<S> {
    cache_manager: Arc<CacheManager>,
    connector_manager: Arc<ConnectorManager>,
    subscribe_manager: Arc<SubscribeManager>,
    schema_manager: Arc<SchemaRegisterManager>,
    client_pool: Arc<ClientPool>,
    message_storage_adapter: Arc<S>,
}

impl<S> GrpcInnerServices<S> {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        subscribe_manager: Arc<SubscribeManager>,
        connector_manager: Arc<ConnectorManager>,
        schema_manager: Arc<SchemaRegisterManager>,
        client_pool: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
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
impl<S> MqttBrokerInnerService for GrpcInnerServices<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    async fn update_cache(
        &self,
        request: Request<UpdateMqttCacheRequest>,
    ) -> Result<Response<UpdateMqttCacheReply>, Status> {
        let req = request.into_inner();
        let conf = broker_mqtt_conf();
        if conf.cluster_name != req.cluster_name {
            return Ok(Response::new(UpdateMqttCacheReply::default()));
        }
        update_cache_metadata(
            &self.cache_manager,
            &self.connector_manager,
            &self.subscribe_manager,
            &self.schema_manager,
            req,
        )
        .await;
        return Ok(Response::new(UpdateMqttCacheReply::default()));
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<DeleteSessionReply>, Status> {
        let req = request.into_inner();
        debug!("Received request from Placement center to delete expired Session. Cluster name :{}, clientId: {:?}",req.cluster_name,req.client_id);
        if self.cache_manager.cluster_name != req.cluster_name {
            return Err(Status::cancelled("Cluster name does not match".to_string()));
        }

        if req.client_id.is_empty() {
            return Err(Status::cancelled("Client ID cannot be empty".to_string()));
        }

        for client_id in req.client_id {
            self.subscribe_manager.remove_client_id(&client_id);
            self.cache_manager.remove_session(&client_id);
        }

        return Ok(Response::new(DeleteSessionReply::default()));
    }

    async fn send_last_will_message(
        &self,
        request: Request<SendLastWillMessageRequest>,
    ) -> Result<Response<SendLastWillMessageReply>, Status> {
        let req = request.into_inner();
        let data = match serde_json::from_slice::<LastWillData>(&req.last_will_message) {
            Ok(da) => da,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        info!(
            "Received will message from placement center, source client id: {},data:{:?}",
            req.client_id, data
        );

        match send_last_will_message(
            &req.client_id,
            &self.cache_manager,
            &self.client_pool,
            &data.last_will,
            &data.last_will_properties,
            self.message_storage_adapter.clone(),
        )
        .await
        {
            Ok(()) => {
                return Ok(Response::new(SendLastWillMessageReply::default()));
            }
            Err(e) => {
                return Err(Status::internal(e.to_string()));
            }
        }
    }
}
