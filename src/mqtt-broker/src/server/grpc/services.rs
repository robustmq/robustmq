// Copyright 2023 RobustMQ Team
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


use crate::handler::cache_manager::CacheManager;
use crate::handler::lastwill::send_last_will_message;
use crate::subscribe::subscribe_manager::SubscribeManager;
use clients::poll::ClientPool;
use metadata_struct::mqtt::lastwill::LastWillData;
use protocol::broker_server::generate::mqtt::{
    mqtt_broker_service_server::MqttBrokerService, CommonReply, UpdateCacheRequest,
};
use protocol::broker_server::generate::mqtt::{DeleteSessionRequest, SendLastWillMessageRequest};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tonic::{Request, Response, Status};

pub struct GrpcBrokerServices<S> {
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_poll: Arc<ClientPool>,
    message_storage_adapter: Arc<S>,
}

impl<S> GrpcBrokerServices<S> {
    pub fn new(
        metadata_cache: Arc<CacheManager>,
        subscribe_manager: Arc<SubscribeManager>,
        client_poll: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
    ) -> Self {
        return GrpcBrokerServices {
            cache_manager: metadata_cache,
            subscribe_manager,
            client_poll,
            message_storage_adapter,
        };
    }
}

#[tonic::async_trait]
impl<S> MqttBrokerService for GrpcBrokerServices<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    async fn update_cache(
        &self,
        request: Request<UpdateCacheRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        return Ok(Response::new(CommonReply::default()));
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        if self.cache_manager.cluster_name != req.cluster_name {
            return Err(Status::cancelled("Cluster name does not match".to_string()));
        }

        if req.client_id.is_empty() {
            return Err(Status::cancelled("Client ID cannot be empty".to_string()));
        }
        for client_id in req.client_id {
            self.cache_manager.remove_session(&client_id);
            self.subscribe_manager.stop_push_by_client_id(&client_id);
        }

        return Ok(Response::new(CommonReply::default()));
    }

    async fn send_last_will_message(
        &self,
        request: Request<SendLastWillMessageRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = match serde_json::from_slice::<LastWillData>(&req.last_will_message) {
            Ok(da) => da,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };
        match send_last_will_message(
            &req.client_id,
            &self.cache_manager,
            &self.client_poll,
            &data.last_will,
            &data.last_will_properties,
            self.message_storage_adapter.clone(),
        )
        .await
        {
            Ok(()) => {
                return Ok(Response::new(CommonReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
