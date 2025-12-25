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

use common_base::error::{common::CommonError, ResultCommonError};
use mqtt_broker::{
    broker::MqttBrokerServerParams, handler::dynamic_cache::update_mqtt_cache_metadata,
};
use protocol::broker::broker_common::{
    broker_common_service_server::BrokerCommonService, BrokerUpdateCacheResourceType,
    UpdateCacheReply, UpdateCacheRequest,
};
use storage_engine::{core::dynamic_cache::update_storage_cache_metadata, StorageEngineParams};
use tonic::{Request, Response, Status};

pub struct GrpcBrokerCommonService {
    mqtt_params: MqttBrokerServerParams,
    storage_params: StorageEngineParams,
}

impl GrpcBrokerCommonService {
    pub fn new(mqtt_params: MqttBrokerServerParams, storage_params: StorageEngineParams) -> Self {
        GrpcBrokerCommonService {
            mqtt_params,
            storage_params,
        }
    }
}

#[tonic::async_trait]
impl BrokerCommonService for GrpcBrokerCommonService {
    async fn update_cache(
        &self,
        request: Request<UpdateCacheRequest>,
    ) -> Result<Response<UpdateCacheReply>, Status> {
        let req = request.into_inner();
        if let Err(e) = update_cache(&self.mqtt_params, &self.storage_params, &req).await {
            return Err(Status::internal(e.to_string()));
        }
        Ok(Response::new(UpdateCacheReply::default()))
    }
}

async fn update_cache(
    mqtt_params: &MqttBrokerServerParams,
    storage_params: &StorageEngineParams,
    req: &UpdateCacheRequest,
) -> ResultCommonError {
    match req.resource_type() {
        BrokerUpdateCacheResourceType::Session
        | BrokerUpdateCacheResourceType::User
        | BrokerUpdateCacheResourceType::Subscribe
        | BrokerUpdateCacheResourceType::Topic
        | BrokerUpdateCacheResourceType::Connector
        | BrokerUpdateCacheResourceType::Schema
        | BrokerUpdateCacheResourceType::SchemaResource => {
            if let Err(e) = update_mqtt_cache_metadata(
                &mqtt_params.cache_manager,
                &mqtt_params.connector_manager,
                &mqtt_params.subscribe_manager,
                &mqtt_params.schema_manager,
                &mqtt_params.message_storage_adapter,
                &mqtt_params.metrics_cache_manager,
                req,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
        BrokerUpdateCacheResourceType::ClusterResourceConfig => {}
        BrokerUpdateCacheResourceType::Node => {}
        BrokerUpdateCacheResourceType::Shard
        | BrokerUpdateCacheResourceType::Segment
        | BrokerUpdateCacheResourceType::SegmentMeta => {
            if let Err(e) = update_storage_cache_metadata(&storage_params.cache_manager, req).await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }
    Ok(())
}
