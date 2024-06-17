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
    cache::{cluster::ClusterCache, journal::JournalCache},
    storage::rocksdb::RocksDBEngine,
};
use bincode::deserialize;
use common_base::errors::RobustMQError;
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
        cluster_cache: Arc<ClusterCache>,
        engine_cache: Arc<JournalCache>,
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
    pub fn route(&self, data: Vec<u8>) -> Result<(), RobustMQError> {
        let storage_data: StorageData = deserialize(data.as_ref()).unwrap();
        match storage_data.data_type {
            StorageDataType::ClusterRegisterNode => {
                return self.route_cluster.cluster_register_node(storage_data.value);
            }
            StorageDataType::ClusterUngisterNode => {
                return self
                    .route_cluster
                    .cluster_unregister_node(storage_data.value);
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
        }
    }
}
