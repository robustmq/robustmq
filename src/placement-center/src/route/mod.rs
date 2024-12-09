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

pub mod apply;
pub mod cluster;
pub mod data;
pub mod journal;
pub mod kv;
pub mod mqtt;

use std::sync::Arc;
use std::time::Instant;

use bincode::{deserialize, serialize};
use common_base::utils::vec_util;
use data::{StorageData, StorageDataType};
use log::{error, info};

use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::journal::cache::JournalCacheManager;
use crate::route::cluster::DataRouteCluster;
use crate::route::journal::DataRouteJournal;
use crate::route::kv::DataRouteKv;
use crate::route::mqtt::DataRouteMqtt;
use crate::storage::rocksdb::{RocksDBEngine, DB_COLUMN_FAMILY_CLUSTER};

#[derive(Clone)]
pub struct DataRoute {
    route_kv: DataRouteKv,
    route_mqtt: DataRouteMqtt,
    route_journal: DataRouteJournal,
    route_cluster: DataRouteCluster,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl DataRoute {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cluster_cache: Arc<PlacementCacheManager>,
        engine_cache: Arc<JournalCacheManager>,
    ) -> DataRoute {
        let route_kv = DataRouteKv::new(rocksdb_engine_handler.clone());
        let route_mqtt = DataRouteMqtt::new(rocksdb_engine_handler.clone());
        let route_cluster =
            DataRouteCluster::new(rocksdb_engine_handler.clone(), cluster_cache.clone());
        let route_journal =
            DataRouteJournal::new(rocksdb_engine_handler.clone(), engine_cache.clone());
        DataRoute {
            route_kv,
            route_mqtt,
            route_journal,
            route_cluster,
            rocksdb_engine_handler,
        }
    }

    //Receive write operations performed by the Raft state machine and write subsequent service data after Raft state machine synchronization is complete.
    pub async fn route(
        &self,
        storage_data: StorageData,
    ) -> Result<Option<Vec<u8>>, PlacementCenterError> {
        match storage_data.data_type {
            // Placement Center
            StorageDataType::KvSet => {
                self.route_kv.set(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::KvDelete => {
                self.route_kv.delete(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::ClusterAddNode => {
                self.route_cluster.add_node(storage_data.value).await?;
                Ok(None)
            }
            StorageDataType::ClusterDeleteNode => {
                self.route_cluster.delete_node(storage_data.value).await?;
                Ok(None)
            }

            StorageDataType::ClusterAddCluster => {
                self.route_cluster.add_cluster(storage_data.value).await?;
                Ok(None)
            }
            StorageDataType::ClusterDeleteCluster => Ok(None),

            StorageDataType::ClusterSetResourceConfig => {
                self.route_cluster.set_resource_config(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::ClusterDeleteResourceConfig => {
                self.route_cluster
                    .delete_resource_config(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::ClusterSetIdempotentData => {
                self.route_cluster.set_idempotent_data(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::ClusterDeleteIdempotentData => {
                self.route_cluster
                    .delete_idempotent_data(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::ClusterSaveOffset => {
                self.route_cluster.save_offset_data(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::ClusterDeleteOffset => {
                self.route_cluster.delete_offset_data(storage_data.value)?;
                Ok(None)
            }

            // Journal Engine
            StorageDataType::JournalSetShard => Ok(Some(
                self.route_journal.set_shard(storage_data.value).await?,
            )),
            StorageDataType::JournalDeleteShard => {
                self.route_journal.delete_shard(storage_data.value).await?;
                Ok(None)
            }
            StorageDataType::JournalSetSegment => Ok(Some(
                self.route_journal.set_segment(storage_data.value).await?,
            )),
            StorageDataType::JournalDeleteSegment => {
                self.route_journal
                    .delete_segment(storage_data.value)
                    .await?;
                Ok(None)
            }

            StorageDataType::JournalSetSegmentMetadata => Ok(Some(
                self.route_journal
                    .set_segment_meta(storage_data.value)
                    .await?,
            )),
            StorageDataType::JournalDeleteSegmentMetadata => {
                self.route_journal
                    .delete_segment_meta(storage_data.value)
                    .await?;
                Ok(None)
            }

            // Mqtt Broker
            StorageDataType::MqttSetAcl => {
                self.route_cluster.create_acl(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttDeleteAcl => {
                self.route_cluster.delete_acl(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttSetBlacklist => {
                self.route_cluster.create_blacklist(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttDeleteBlacklist => {
                self.route_cluster.delete_blacklist(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttSetUser => {
                self.route_mqtt.create_user(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttDeleteUser => {
                self.route_mqtt.delete_user(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttSetTopic => {
                self.route_mqtt.create_topic(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttDeleteTopic => {
                self.route_mqtt.delete_topic(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttSetSession => {
                self.route_mqtt.create_session(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttDeleteSession => {
                self.route_mqtt.delete_session(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttUpdateSession => {
                self.route_mqtt.update_session(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttSaveLastWillMessage => {
                self.route_mqtt.save_last_will_message(storage_data.value)?;
                Ok(None)
            }
            StorageDataType::MqttSetNxExclusiveTopic => {
                let is_set = self.route_mqtt.set_nx_exclusive_topic(storage_data.value)?;
                Ok(Some(vec_util::bool_to_vec(is_set)))
            }
            StorageDataType::MqttDeleteExclusiveTopic => {
                self.route_mqtt.delete_exclusive_topic(storage_data.value)?;
                Ok(None)
            }
        }
    }

    pub fn build_snapshot(&self) -> Vec<u8> {
        info!("Start building snapshots");
        let cf = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_CLUSTER)
        {
            cf
        } else {
            error!(
                "{}",
                PlacementCenterError::RocksDBFamilyNotAvailable(
                    DB_COLUMN_FAMILY_CLUSTER.to_string(),
                )
            );
            return Vec::new();
        };

        let res = match self.rocksdb_engine_handler.read_all_by_cf(cf) {
            Ok(data) => data,
            Err(e) => {
                error!("{}", e.to_string());
                return Vec::new();
            }
        };

        let res = match serialize(&res) {
            Ok(data) => data,
            Err(e) => {
                error!("{}", e.to_string());
                return Vec::new();
            }
        };
        info!("Snapshot built successfully, snapshot size :{}", res.len());
        res
    }

    pub fn recover_snapshot(&self, data: Vec<u8>) -> Result<(), PlacementCenterError> {
        info!("Start restoring snapshot, snapshot length :{}", data.len());
        let now = Instant::now();
        let records = match deserialize::<Vec<(String, Vec<u8>)>>(&data) {
            Ok(data) => data,
            Err(e) => {
                return Err(PlacementCenterError::CommonError(e.to_string()));
            }
        };

        let cf = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_CLUSTER)
        {
            cf
        } else {
            return Err(PlacementCenterError::RocksDBFamilyNotAvailable(
                DB_COLUMN_FAMILY_CLUSTER.to_string(),
            ));
        };

        for raw in records {
            self.rocksdb_engine_handler.write(cf, &raw.0, &raw.1)?;
        }

        info!(
            "Snapshot recovery was successful, snapshot size {}, time: {}",
            data.len(),
            now.elapsed().as_millis()
        );
        Ok(())
    }
}
