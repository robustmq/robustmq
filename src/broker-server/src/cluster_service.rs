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
    UpdateCacheRecord, UpdateCacheReply, UpdateCacheRequest,
};
use storage_engine::{core::dynamic_cache::update_storage_cache_metadata, StorageEngineParams};
use tonic::{Request, Response, Status};
use tracing::warn;

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
        for record in req.records.iter() {
            if let Err(e) = update_cache(&self.mqtt_params, &self.storage_params, record).await {
                warn!(
                    "Failed to update cache for resource type {:?}, action: {:?}, error: {:?}",
                    record.resource_type(), record.action_type(), e
                );
            }
        }

        Ok(Response::new(UpdateCacheReply::default()))
    }
}

async fn update_cache(
    mqtt_params: &MqttBrokerServerParams,
    storage_params: &StorageEngineParams,
    record: &UpdateCacheRecord,
) -> ResultCommonError {
    match record.resource_type() {
        // MQTT Broker
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
                &mqtt_params.storage_driver_manager,
                &mqtt_params.metrics_cache_manager,
                record,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }

        // Cluster
        BrokerUpdateCacheResourceType::ClusterResourceConfig
        | BrokerUpdateCacheResourceType::Node => {}

        // Storage Engine
        BrokerUpdateCacheResourceType::Shard
        | BrokerUpdateCacheResourceType::Segment
        | BrokerUpdateCacheResourceType::SegmentMeta => {
            if let Err(e) = update_storage_cache_metadata(
                &storage_params.cache_manager,
                &storage_params.rocksdb_engine_handler,
                record,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }
    Ok(())
}
