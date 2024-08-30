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

pub mod cluster;
pub mod journal;
pub mod kv;
pub mod mqtt;

use super::{
    apply::{StorageData, StorageDataType},
    route::{
        cluster::DataRouteCluster, journal::DataRouteJournal, kv::DataRouteKv, mqtt::DataRouteMQTT,
    },
};
use crate::{
    cache::{journal::JournalCacheManager, placement::PlacementCacheManager},
    storage::rocksdb::RocksDBEngine,
};
use bincode::deserialize;
use common_base::error::common::CommonError;
use std::sync::Arc;

pub struct DataRoute {
    route_kv: DataRouteKv,
    route_mqtt: DataRouteMQTT,
    route_journal: DataRouteJournal,
    route_cluster: DataRouteCluster,
}

impl DataRoute {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cluster_cache: Arc<PlacementCacheManager>,
        engine_cache: Arc<JournalCacheManager>,
    ) -> DataRoute {
        let route_kv = DataRouteKv::new(rocksdb_engine_handler.clone());
        let route_mqtt = DataRouteMQTT::new(rocksdb_engine_handler.clone());
        let route_cluster =
            DataRouteCluster::new(rocksdb_engine_handler.clone(), cluster_cache.clone());
        let route_journal = DataRouteJournal::new(
            rocksdb_engine_handler.clone(),
            engine_cache.clone(),
            cluster_cache.clone(),
        );
        return DataRoute {
            route_kv,
            route_mqtt,
            route_journal,
            route_cluster,
        };
    }

    //Receive write operations performed by the Raft state machine and write subsequent service data after Raft state machine synchronization is complete.
    pub fn route(&self, data: Vec<u8>) -> Result<(), CommonError> {
        let storage_data: StorageData = deserialize(data.as_ref()).unwrap();
        match storage_data.data_type {
            StorageDataType::ClusterRegisterNode => {
                return self.route_cluster.add_node(storage_data.value);
            }
            StorageDataType::ClusterUngisterNode => {
                return self.route_cluster.delete_node(storage_data.value);
            }
            StorageDataType::ClusterSetResourceConfig => {
                return self.route_cluster.set_resource_config(storage_data.value);
            }
            StorageDataType::ClusterDeleteResourceConfig => {
                return self
                    .route_cluster
                    .delete_resource_config(storage_data.value);
            }
            StorageDataType::ClusterSetIdempotentData => {
                return self.route_cluster.set_idempotent_data(storage_data.value);
            }
            StorageDataType::ClusterDeleteIdempotentData => {
                return self
                    .route_cluster
                    .delete_idempotent_data(storage_data.value);
            }
            StorageDataType::MQTTCreateAcl => {
                return self.route_cluster.create_acl(storage_data.value);
            }
            StorageDataType::MQTTDeleteAcl => {
                return self.route_cluster.delete_acl(storage_data.value);
            }
            StorageDataType::MQTTCreateBlacklist => {
                return self.route_cluster.create_blacklist(storage_data.value);
            }
            StorageDataType::MQTTDeleteBlacklist => {
                return self.route_cluster.delete_blacklist(storage_data.value);
            }

            StorageDataType::JournalCreateShard => {
                return self.route_journal.create_shard(storage_data.value);
            }
            StorageDataType::JournalDeleteShard => {
                return self.route_journal.delete_shard(storage_data.value);
            }
            StorageDataType::JournalCreateSegment => {
                return self.route_journal.create_segment(storage_data.value);
            }
            StorageDataType::JournalDeleteSegment => {
                return self.route_journal.delete_segment(storage_data.value);
            }
            StorageDataType::KvSet => {
                return self.route_kv.set(storage_data.value);
            }
            StorageDataType::KvDelete => {
                return self.route_kv.delete(storage_data.value);
            }
            StorageDataType::MQTTCreateUser => {
                return self.route_mqtt.create_user(storage_data.value);
            }
            StorageDataType::MQTTDeleteUser => {
                return self.route_mqtt.delete_user(storage_data.value);
            }
            StorageDataType::MQTTCreateTopic => {
                return self.route_mqtt.create_topic(storage_data.value);
            }
            StorageDataType::MQTTDeleteTopic => {
                return self.route_mqtt.delete_topic(storage_data.value);
            }
            StorageDataType::MQTTCreateSession => {
                return self.route_mqtt.create_session(storage_data.value);
            }
            StorageDataType::MQTTDeleteSession => {
                return self.route_mqtt.delete_session(storage_data.value);
            }
            StorageDataType::MQTTUpdateSession => {
                return self.route_mqtt.update_session(storage_data.value);
            }
            StorageDataType::MQTTSetTopicRetainMessage => {
                return self.route_mqtt.set_topic_retain_message(storage_data.value);
            }
            StorageDataType::MQTTSaveLastWillMessage => {
                return self.route_mqtt.save_last_will_message(storage_data.value);
            }
        }
    }
}
