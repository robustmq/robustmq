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

use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::route::common::DataRouteCluster;
use crate::raft::route::journal::DataRouteJournal;
use crate::raft::route::kv::DataRouteKv;
use crate::raft::route::mqtt::DataRouteMqtt;
use bytes::Bytes;
use data::{StorageData, StorageDataType};
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod common;
pub mod data;
pub mod journal;
pub mod kv;
pub mod mqtt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppResponseData {
    pub value: Option<Bytes>,
}

#[derive(Clone)]
pub struct DataRoute {
    route_kv: DataRouteKv,
    route_mqtt: DataRouteMqtt,
    route_journal: DataRouteJournal,
    route_cluster: DataRouteCluster,
}

impl DataRoute {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<CacheManager>,
    ) -> DataRoute {
        let route_kv = DataRouteKv::new(rocksdb_engine_handler.clone());
        let route_mqtt = DataRouteMqtt::new(rocksdb_engine_handler.clone(), cache_manager.clone());
        let route_cluster =
            DataRouteCluster::new(rocksdb_engine_handler.clone(), cache_manager.clone());
        let route_journal = DataRouteJournal::new(rocksdb_engine_handler, cache_manager);
        DataRoute {
            route_kv,
            route_mqtt,
            route_journal,
            route_cluster,
        }
    }

    //Receive write operations performed by the Raft state machine and write subsequent service data after Raft state machine synchronization is complete.
    pub async fn route(
        &self,
        storage_data: &StorageData,
    ) -> Result<Option<Bytes>, MetaServiceError> {
        match storage_data.data_type {
            // Meta Service
            StorageDataType::KvSet => {
                self.route_kv.set(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::KvDelete => {
                self.route_kv.delete(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::ClusterAddNode => {
                self.route_cluster
                    .add_node(storage_data.value.clone())
                    .await?;
                Ok(None)
            }
            StorageDataType::ClusterDeleteNode => {
                self.route_cluster
                    .delete_node(storage_data.value.clone())
                    .await?;
                Ok(None)
            }

            StorageDataType::ClusterAddCluster => {
                self.route_cluster
                    .add_cluster(storage_data.value.clone())
                    .await?;
                Ok(None)
            }
            StorageDataType::ClusterDeleteCluster => Ok(None),

            StorageDataType::ResourceConfigSet => {
                self.route_cluster
                    .set_resource_config(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::ResourceConfigDelete => {
                self.route_cluster
                    .delete_resource_config(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::OffsetSet => {
                self.route_cluster
                    .save_offset_data(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::OffsetDelete => {
                self.route_cluster
                    .delete_offset_data(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::SchemaSet => {
                self.route_cluster.set_schema(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::SchemaDelete => {
                self.route_cluster
                    .delete_schema(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::SchemaBindSet => {
                self.route_cluster
                    .set_schema_bind(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::SchemaBindDelete => {
                self.route_cluster
                    .delete_schema_bind(storage_data.value.clone())?;
                Ok(None)
            }

            // Journal Engine
            StorageDataType::JournalSetShard => Ok(Some(
                self.route_journal
                    .set_shard(storage_data.value.clone())
                    .await?,
            )),
            StorageDataType::JournalDeleteShard => {
                self.route_journal
                    .delete_shard(storage_data.value.clone())
                    .await?;
                Ok(None)
            }
            StorageDataType::JournalSetSegment => Ok(Some(
                self.route_journal
                    .set_segment(storage_data.value.clone())
                    .await?,
            )),
            StorageDataType::JournalDeleteSegment => {
                self.route_journal
                    .delete_segment(storage_data.value.clone())
                    .await?;
                Ok(None)
            }

            StorageDataType::JournalSetSegmentMetadata => Ok(Some(
                self.route_journal
                    .set_segment_meta(storage_data.value.clone())
                    .await?,
            )),
            StorageDataType::JournalDeleteSegmentMetadata => {
                self.route_journal
                    .delete_segment_meta(storage_data.value.clone())
                    .await?;
                Ok(None)
            }

            // Mqtt Broker
            StorageDataType::MqttSetAcl => {
                self.route_mqtt.create_acl(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttDeleteAcl => {
                self.route_mqtt.delete_acl(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttSetBlacklist => {
                self.route_mqtt
                    .create_blacklist(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttDeleteBlacklist => {
                self.route_mqtt
                    .delete_blacklist(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttSetUser => {
                self.route_mqtt.create_user(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttDeleteUser => {
                self.route_mqtt.delete_user(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttSetTopic => {
                self.route_mqtt.create_topic(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttDeleteTopic => {
                self.route_mqtt.delete_topic(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttSetRetainMessage => {
                self.route_mqtt
                    .set_retain_message(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttDeleteRetainMessage => {
                self.route_mqtt
                    .delete_retain_message(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttSetSession => {
                self.route_mqtt.create_session(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttDeleteSession => {
                self.route_mqtt.delete_session(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttUpdateSession => {
                self.route_mqtt.update_session(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttSaveLastWillMessage => {
                self.route_mqtt
                    .save_last_will_message(storage_data.value.clone())?;
                Ok(None)
            }

            StorageDataType::MqttCreateTopicRewriteRule => {
                self.route_mqtt
                    .create_topic_rewrite_rule(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttDeleteTopicRewriteRule => {
                self.route_mqtt
                    .delete_topic_rewrite_rule(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttSetSubscribe => {
                self.route_mqtt.set_subscribe(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttDeleteSubscribe => {
                self.route_mqtt
                    .delete_subscribe(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttSetConnector => {
                self.route_mqtt.set_connector(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttDeleteConnector => {
                self.route_mqtt
                    .delete_connector(storage_data.value.clone())?;
                Ok(None)
            }

            // auto subscribe
            StorageDataType::MqttSetAutoSubscribeRule => {
                self.route_mqtt
                    .set_auto_subscribe_rule(storage_data.value.clone())?;
                Ok(None)
            }
            StorageDataType::MqttDeleteAutoSubscribeRule => {
                self.route_mqtt
                    .delete_auto_subscribe_rule(storage_data.value.clone())?;
                Ok(None)
            }
        }
    }
}
