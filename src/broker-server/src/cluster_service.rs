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

use crate::update_cache::update_cache;
use mqtt_broker::{
    broker::MqttBrokerServerParams, core::inner::send_last_will_message_by_req,
    core::qos::get_qos_data_by_req,
};
use nats_broker::broker::NatsBrokerServerParams;
use protocol::broker::broker::{
    broker_service_server::BrokerService, GetQosDataByClientIdReply, GetQosDataByClientIdRequest,
    SendLastWillMessageReply, SendLastWillMessageRequest, UpdateCacheReply, UpdateCacheRequest,
};
use storage_engine::StorageEngineParams;
use tonic::{Request, Response, Status};
use tracing::warn;

pub struct GrpcBrokerService {
    mqtt_params: MqttBrokerServerParams,
    nats_params: NatsBrokerServerParams,
    storage_params: StorageEngineParams,
}

impl GrpcBrokerService {
    pub fn new(
        mqtt_params: MqttBrokerServerParams,
        nats_params: NatsBrokerServerParams,
        storage_params: StorageEngineParams,
    ) -> Self {
        GrpcBrokerService {
            mqtt_params,
            nats_params,
            storage_params,
        }
    }
}

#[tonic::async_trait]
impl BrokerService for GrpcBrokerService {
    async fn update_cache(
        &self,
        request: Request<UpdateCacheRequest>,
    ) -> Result<Response<UpdateCacheReply>, Status> {
        let req = request.into_inner();
        for record in req.records.iter() {
            if let Err(e) = update_cache(
                &self.mqtt_params,
                &self.nats_params,
                &self.storage_params,
                record,
            )
            .await
            {
                warn!(
                    "Failed to update cache for resource type {:?}, action: {:?}, error: {:?}",
                    record.resource_type(),
                    record.action_type(),
                    e
                );
            }
        }

        Ok(Response::new(UpdateCacheReply::default()))
    }

    async fn send_last_will_message(
        &self,
        request: Request<SendLastWillMessageRequest>,
    ) -> Result<Response<SendLastWillMessageReply>, Status> {
        let req = request.into_inner();
        send_last_will_message_by_req(
            &self.mqtt_params.cache_manager,
            &self.mqtt_params.client_pool,
            &self.mqtt_params.retain_message_manager,
            &self.mqtt_params.storage_driver_manager,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn get_qos_data_by_client_id(
        &self,
        request: Request<GetQosDataByClientIdRequest>,
    ) -> Result<Response<GetQosDataByClientIdReply>, Status> {
        let req = request.into_inner();
        get_qos_data_by_req(&self.mqtt_params.cache_manager, &req.client_ids)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }
}
