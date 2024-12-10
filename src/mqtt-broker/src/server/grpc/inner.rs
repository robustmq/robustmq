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

use grpc_clients::pool::ClientPool;
use log::debug;
use metadata_struct::mqtt::lastwill::LastWillData;
use protocol::broker_mqtt::broker_mqtt_inner::mqtt_broker_inner_service_server::MqttBrokerInnerService;
use protocol::broker_mqtt::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateCacheReply, UpdateCacheRequest,
};
use storage_adapter::storage::StorageAdapter;
use tonic::{Request, Response, Status};

use crate::handler::cache::{update_cache_metadata, CacheManager};
use crate::handler::lastwill::send_last_will_message;
use crate::subscribe::subscribe_manager::SubscribeManager;

pub struct GrpcInnerServices<S> {
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_pool: Arc<ClientPool>,
    message_storage_adapter: Arc<S>,
}

impl<S> GrpcInnerServices<S> {
    pub fn new(
        metadata_cache: Arc<CacheManager>,
        subscribe_manager: Arc<SubscribeManager>,
        client_pool: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
    ) -> Self {
        GrpcInnerServices {
            cache_manager: metadata_cache,
            subscribe_manager,
            client_pool,
            message_storage_adapter,
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
        request: Request<UpdateCacheRequest>,
    ) -> Result<Response<UpdateCacheReply>, Status> {
        let req = request.into_inner();
        update_cache_metadata(req);
        return Ok(Response::new(UpdateCacheReply::default()));
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
            self.subscribe_manager
                .remove_exclusive_subscribe_by_client_id(&client_id)
                .await?;
            self.cache_manager.remove_session(&client_id);
            self.subscribe_manager.stop_push_by_client_id(&client_id);
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
        debug!(
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
