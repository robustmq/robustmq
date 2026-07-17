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
use amqp_broker::core::cache::AmqpCacheManager;
use kafka_broker::core::cache::KafkaCacheManager;
use metadata_struct::storage::record::StorageRecord;
use mqtt_broker::{
    broker::MqttBrokerServerParams, core::inner::send_last_will_message_by_req,
    core::qos::get_qos_data_by_req,
};
use nats_broker::broker::NatsBrokerServerParams;
use nats_broker::push::nats_fanout::send_packet;
use protocol::broker::broker::{
    broker_service_server::BrokerService, GetQosDataByClientIdReply, GetQosDataByClientIdRequest,
    GetShardSegmentDeleteStatusReply, GetShardSegmentDeleteStatusRequest, QueryReplicaLeoReply,
    QueryReplicaLeoRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    SendNatsShareGroupMessageReply, SendNatsShareGroupMessageRequest, ShardSegmentDeleteStatus,
    UpdateCacheReply, UpdateCacheRequest,
};
use std::sync::Arc;
use storage_engine::core::delete::{segment_already_delete, shard_already_delete};
use storage_engine::isr::handle_epoch::query_local_replica_state;
use storage_engine::isr::handle_fetch::FetchEngines;
use storage_engine::StorageEngineParams;
use tonic::{Request, Response, Status};
use tracing::warn;

pub struct GrpcBrokerService {
    mqtt_params: MqttBrokerServerParams,
    nats_params: NatsBrokerServerParams,
    storage_params: StorageEngineParams,
    kafka_cache: Arc<KafkaCacheManager>,
    amqp_cache: Arc<AmqpCacheManager>,
}

impl GrpcBrokerService {
    pub fn new(
        mqtt_params: MqttBrokerServerParams,
        nats_params: NatsBrokerServerParams,
        storage_params: StorageEngineParams,
        kafka_cache: Arc<KafkaCacheManager>,
        amqp_cache: Arc<AmqpCacheManager>,
    ) -> Self {
        GrpcBrokerService {
            mqtt_params,
            nats_params,
            storage_params,
            kafka_cache,
            amqp_cache,
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
                &self.kafka_cache,
                &self.amqp_cache,
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

    async fn get_shard_segment_delete_status(
        &self,
        request: Request<GetShardSegmentDeleteStatusRequest>,
    ) -> Result<Response<GetShardSegmentDeleteStatusReply>, Status> {
        let req = request.into_inner();
        let mut results = Vec::with_capacity(req.items.len());

        for item in &req.items {
            let deleted = if let Some(segment_seq) = item.segment_seq {
                segment_already_delete(
                    &self.storage_params.cache_manager,
                    &item.shard_name,
                    segment_seq,
                )
            } else {
                shard_already_delete(&self.storage_params.cache_manager, &item.shard_name)
            };

            results.push(ShardSegmentDeleteStatus {
                shard_name: item.shard_name.clone(),
                deleted,
            });
        }

        Ok(Response::new(GetShardSegmentDeleteStatusReply { results }))
    }

    async fn send_nats_share_group_message(
        &self,
        request: Request<SendNatsShareGroupMessageRequest>,
    ) -> Result<Response<SendNatsShareGroupMessageReply>, Status> {
        let req = request.into_inner();
        if let Some(subscribe) = self
            .nats_params
            .subscribe_manager
            .get_subscribe(req.connect_id, &req.sid)
        {
            let record =
                StorageRecord::decode(&req.record).map_err(|e| Status::internal(e.to_string()))?;
            send_packet(
                &self.nats_params.connection_manager,
                subscribe.connect_id,
                &subscribe.subject,
                &subscribe.sid,
                &record,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
            return Ok(Response::new(SendNatsShareGroupMessageReply {}));
        }
        Err(Status::not_found(format!(
            "subscriber not found: connect_id={}, sid={}",
            req.connect_id, req.sid
        )))
    }

    async fn query_replica_leo(
        &self,
        request: Request<QueryReplicaLeoRequest>,
    ) -> Result<Response<QueryReplicaLeoReply>, Status> {
        let req = request.into_inner();
        let engines = FetchEngines {
            memory: self.storage_params.memory_storage_engine.clone(),
            rocksdb: self.storage_params.rocksdb_storage_engine.clone(),
            segment: Arc::new(
                storage_engine::filesegment::replica::FileSegmentReplicaLog::new(
                    self.storage_params.cache_manager.clone(),
                    self.storage_params.rocksdb_engine_handler.clone(),
                ),
            ),
        };
        let state = query_local_replica_state(
            &engines,
            &self.storage_params.cache_manager,
            &req.shard_name,
            req.segment_seq,
        );
        Ok(Response::new(QueryReplicaLeoReply {
            segment_leo: state.segment_leo,
            latest_leader_epoch: state.latest_leader_epoch,
            log_start_offset: state.log_start_offset,
            available: state.available,
        }))
    }
}
