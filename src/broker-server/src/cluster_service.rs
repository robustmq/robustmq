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

use crate::dynamic_cache::update_cluster_cache_metadata;
use common_base::error::{common::CommonError, ResultCommonError};
use metadata_struct::topic::TopicSource;
use mqtt_broker::{
    broker::MqttBrokerServerParams,
    core::{dynamic_cache::update_mqtt_cache_metadata, topic::delete_topic_by_mqtt},
};
use nats_broker::{
    broker::NatsBrokerServerParams, core::dynamic_cache::update_nats_cache_metadata,
};
use protocol::broker::broker_common::{
    broker_common_service_server::BrokerCommonService, BatchDeleteGroupsReply,
    BatchDeleteGroupsRequest, BatchDeleteTopicsReply, BatchDeleteTopicsRequest,
    BrokerUpdateCacheResourceType, UpdateCacheRecord, UpdateCacheReply, UpdateCacheRequest,
};
use storage_engine::{core::dynamic_cache::update_storage_cache_metadata, StorageEngineParams};
use tonic::{Request, Response, Status};
use tracing::warn;

pub struct GrpcBrokerCommonService {
    mqtt_params: MqttBrokerServerParams,
    nats_params: NatsBrokerServerParams,
    storage_params: StorageEngineParams,
}

impl GrpcBrokerCommonService {
    pub fn new(
        mqtt_params: MqttBrokerServerParams,
        nats_params: NatsBrokerServerParams,
        storage_params: StorageEngineParams,
    ) -> Self {
        GrpcBrokerCommonService {
            mqtt_params,
            nats_params,
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

    async fn batch_delete_topics(
        &self,
        request: Request<BatchDeleteTopicsRequest>,
    ) -> Result<Response<BatchDeleteTopicsReply>, Status> {
        for topic_info in request.into_inner().topics.iter() {
            let topic = if let Some(topic) = self
                .mqtt_params
                .node_cache
                .get_topic_by_name(&topic_info.tenant, &topic_info.topic_name)
            {
                topic.clone()
            } else {
                continue;
            };

            self.mqtt_params
                .node_cache
                .delete_topic(&topic.tenant, &topic.topic_name);

            match topic.source {
                TopicSource::SystemInner => {
                    // system Topic is not allowed to be deleted. Ignore the logic
                }
                TopicSource::MQTT => {
                    let _res = delete_topic_by_mqtt(
                        &self.mqtt_params.cache_manager,
                        &topic,
                        &self.mqtt_params.storage_driver_manager,
                        &self.mqtt_params.subscribe_manager,
                    )
                    .await;
                }
                TopicSource::NATS => {}
                TopicSource::MQ9 => {}
                TopicSource::Kafka => {}
                TopicSource::AMQP => {}
            }
        }
        Ok(Response::new(BatchDeleteTopicsReply::default()))
    }

    async fn batch_delete_groups(
        &self,
        _request: Request<BatchDeleteGroupsRequest>,
    ) -> Result<Response<BatchDeleteGroupsReply>, Status> {
        Ok(Response::new(BatchDeleteGroupsReply::default()))
    }
}

async fn update_cache(
    mqtt_params: &MqttBrokerServerParams,
    nats_params: &NatsBrokerServerParams,
    storage_params: &StorageEngineParams,
    record: &UpdateCacheRecord,
) -> ResultCommonError {
    match record.resource_type() {
        // MQTT Broker
        BrokerUpdateCacheResourceType::Session
        | BrokerUpdateCacheResourceType::Subscribe
        | BrokerUpdateCacheResourceType::Topic
        | BrokerUpdateCacheResourceType::Connector
        | BrokerUpdateCacheResourceType::Schema
        | BrokerUpdateCacheResourceType::SchemaResource
        | BrokerUpdateCacheResourceType::AutoSubscribeRule
        | BrokerUpdateCacheResourceType::TopicRewriteRule => {
            if let Err(e) = update_mqtt_cache_metadata(
                &mqtt_params.cache_manager,
                &mqtt_params.connector_manager,
                &mqtt_params.subscribe_manager,
                &mqtt_params.schema_manager,
                &mqtt_params.storage_driver_manager,
                &mqtt_params.metrics_cache_manager,
                &mqtt_params.security_manager,
                record,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }

        // Cluster — Node, Config, Tenant, User, Acl, Blacklist
        BrokerUpdateCacheResourceType::ClusterResourceConfig
        | BrokerUpdateCacheResourceType::Node
        | BrokerUpdateCacheResourceType::Tenant
        | BrokerUpdateCacheResourceType::User
        | BrokerUpdateCacheResourceType::Acl
        | BrokerUpdateCacheResourceType::Blacklist => {
            if let Err(e) = update_cluster_cache_metadata(
                &mqtt_params.cache_manager.node_cache,
                &mqtt_params.security_manager,
                record,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }

        // NATS / MQ9
        BrokerUpdateCacheResourceType::NatsSubscribe | BrokerUpdateCacheResourceType::Mq9Email => {
            if let Err(e) = update_nats_cache_metadata(
                &nats_params.cache_manager,
                &nats_params.subscribe_manager,
                record,
            )
            .await
            {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }

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
