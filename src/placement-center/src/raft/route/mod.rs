pub mod cluster;
pub mod mqtt;
pub mod journal;
pub mod kv;

use super::{
    route::{
        cluster::DataRouteCluster, journal::DataRouteJournal, kv::DataRouteKv, mqtt::DataRouteMQTT,
    },
    apply::{StorageData, StorageDataType},
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
            StorageDataType::RegisterNode => {
                return self.route_cluster.cluster_register_node(storage_data.value);
            }
            StorageDataType::UngisterNode => {
                return self
                    .route_cluster
                    .cluster_unregister_node(storage_data.value);
            }
            StorageDataType::CreateShard => {
                return self.route_journal.create_shard(storage_data.value);
            }
            StorageDataType::DeleteShard => {
                return self.route_journal.delete_shard(storage_data.value);
            }
            StorageDataType::CreateSegment => {
                return self.route_journal.create_segment(storage_data.value);
            }
            StorageDataType::DeleteSegment => {
                return self.route_journal.delete_segment(storage_data.value);
            }
            StorageDataType::Set => {
                return self.route_kv.set(storage_data.value);
            }
            StorageDataType::Delete => {
                return self.route_kv.delete(storage_data.value);
            }
            StorageDataType::MQTTCreateUser => {
                return self.route_mqtt.create_user(storage_data.value);
            }
            StorageDataType::MQTTDeleteUser => {
                return self.route_mqtt.delete_user(storage_data.value);
            }
        }
    }
}
