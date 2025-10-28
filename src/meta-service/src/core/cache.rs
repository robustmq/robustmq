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

use super::heartbeat::NodeHeartbeatData;
use crate::core::error::MetaServiceError;
use crate::server::services::mqtt::connector::ConnectorHeartbeat;
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use crate::storage::journal::shard::ShardStorage;
use crate::storage::mqtt::connector::MqttConnectorStorage;
use crate::storage::mqtt::user::MqttUserStorage;
use crate::storage::placement::cluster::ClusterStorage;
use crate::storage::placement::node::NodeStorage;
use crate::{
    controller::mqtt::session_expire::ExpireLastWill, storage::mqtt::topic::MqttTopicStorage,
};
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::journal::segment::JournalSegment;
use metadata_struct::journal::segment_meta::JournalSegmentMetadata;
use metadata_struct::journal::shard::JournalShard;
use metadata_struct::meta::cluster::ClusterInfo;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::user::MqttUser;
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct CacheManager {
    // (cluster_name, ClusterInfo)
    pub cluster_list: DashMap<String, ClusterInfo>,

    // (cluster_name, (node_id, BrokerNode))
    pub node_list: DashMap<String, DashMap<u64, BrokerNode>>,

    // (cluster_name_node_id, NodeHeartbeatData)
    pub node_heartbeat: DashMap<String, NodeHeartbeatData>,

    // MQTT
    // (cluster_name,(topic_name,topic))
    pub topic_list: DashMap<String, DashMap<String, MQTTTopic>>,

    // (cluster_name,(username,user))
    pub user_list: DashMap<String, DashMap<String, MqttUser>>,

    // (cluster_name,(client_id,ExpireLastWill))
    pub expire_last_wills: DashMap<String, DashMap<String, ExpireLastWill>>,

    // (cluster_name,(client_id,MQTTConnector))
    pub connector_list: DashMap<String, DashMap<String, MQTTConnector>>,

    //(cluster_connector_name, ConnectorHeartbeat)
    pub connector_heartbeat: DashMap<String, ConnectorHeartbeat>,

    // Journal
    //（cluster_name_namespace_shard_name, JournalShard）
    pub shard_list: DashMap<String, JournalShard>,
    pub segment_list: DashMap<String, DashMap<u32, JournalSegment>>,
    pub segment_meta_list: DashMap<String, DashMap<u32, JournalSegmentMetadata>>,
    pub wait_delete_shard_list: DashMap<String, JournalShard>,
    pub wait_delete_segment_list: DashMap<String, JournalSegment>,
}

impl CacheManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> CacheManager {
        let mut cache = CacheManager {
            cluster_list: DashMap::with_capacity(2),
            node_heartbeat: DashMap::with_capacity(2),
            node_list: DashMap::with_capacity(2),
            topic_list: DashMap::with_capacity(8),
            user_list: DashMap::with_capacity(8),
            expire_last_wills: DashMap::with_capacity(8),
            connector_list: DashMap::with_capacity(8),
            connector_heartbeat: DashMap::with_capacity(8),
            shard_list: DashMap::with_capacity(8),
            segment_list: DashMap::with_capacity(256),
            segment_meta_list: DashMap::with_capacity(256),
            wait_delete_shard_list: DashMap::with_capacity(8),
            wait_delete_segment_list: DashMap::with_capacity(8),
        };
        cache.load_cache(rocksdb_engine_handler);
        cache
    }

    // Cluster
    pub fn add_broker_cluster(&self, cluster: &ClusterInfo) {
        self.cluster_list
            .insert(cluster.cluster_name.clone(), cluster.clone());
    }

    pub fn get_cluster(&self, cluster_name: &str) -> Option<ClusterInfo> {
        if let Some(cluster) = self.cluster_list.get(cluster_name) {
            return Some(cluster.clone());
        }
        None
    }

    pub fn get_all_cluster(&self) -> Vec<ClusterInfo> {
        self.cluster_list.iter().map(|row| row.clone()).collect()
    }

    pub fn get_all_cluster_name(&self) -> Vec<String> {
        self.cluster_list
            .iter()
            .map(|row| row.cluster_name.clone())
            .collect()
    }

    // Node
    pub fn add_broker_node(&self, node: BrokerNode) {
        if let Some(data) = self.node_list.get_mut(&node.cluster_name) {
            data.insert(node.node_id, node);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(node.node_id, node.clone());
            self.node_list.insert(node.cluster_name.clone(), data);
        }
    }

    pub fn remove_broker_node(
        &self,
        cluster_name: &str,
        node_id: u64,
    ) -> Option<(u64, BrokerNode)> {
        if let Some(data) = self.node_list.get_mut(cluster_name) {
            return data.remove(&node_id);
        }
        self.remove_broker_heart(cluster_name, node_id);
        None
    }

    pub fn get_broker_num(&self, cluster_name: &str) -> usize {
        if let Some(data) = self.node_list.get(cluster_name) {
            return data.len();
        }
        0
    }

    pub fn get_broker_node(&self, cluster_name: &str, node_id: u64) -> Option<BrokerNode> {
        if let Some(data) = self.node_list.get(cluster_name) {
            if let Some(value) = data.get(&node_id) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn get_broker_node_addr_by_cluster(&self, cluster_name: &str) -> Vec<String> {
        if let Some(data) = self.node_list.get(cluster_name) {
            return data.iter().map(|row| row.node_inner_addr.clone()).collect();
        }
        Vec::new()
    }

    pub fn get_broker_node_id_by_cluster(&self, cluster_name: &str) -> Vec<u64> {
        if let Some(data) = self.node_list.get(cluster_name) {
            return data.iter().map(|row| row.node_id).collect();
        }
        Vec::new()
    }

    pub fn get_broker_node_by_cluster(&self, cluster_name: &str) -> Vec<BrokerNode> {
        if let Some(data) = self.node_list.get(cluster_name) {
            return data.iter().map(|row| row.clone()).collect();
        }
        Vec::new()
    }

    // Heartbeat
    pub fn report_broker_heart(&self, cluster_name: &str, node_id: u64) {
        let key = self.node_key(cluster_name, node_id);
        let data = NodeHeartbeatData {
            cluster_name: cluster_name.to_string(),
            node_id,
            time: now_second(),
        };
        self.node_heartbeat.insert(key, data);
    }

    fn remove_broker_heart(&self, cluster_name: &str, node_id: u64) {
        let key = self.node_key(cluster_name, node_id);
        self.node_heartbeat.remove(&key);
    }

    pub fn get_broker_heart(&self, cluster_name: &str, node_id: u64) -> Option<NodeHeartbeatData> {
        let key = self.node_key(cluster_name, node_id);
        if let Some(heart) = self.node_heartbeat.get(&key) {
            return Some(heart.clone());
        }
        None
    }

    pub fn load_cache(&mut self, rocksdb_engine_handler: Arc<RocksDBEngine>) {
        let cluster = ClusterStorage::new(rocksdb_engine_handler.clone());
        if let Ok(result) = cluster.list() {
            for cluster in result {
                self.add_broker_cluster(&cluster);
            }
        }

        let node = NodeStorage::new(rocksdb_engine_handler.clone());
        if let Ok(result) = node.list(None) {
            for bn in result {
                self.add_broker_node(bn);
            }
        }
    }

    fn node_key(&self, cluster_name: &str, node_id: u64) -> String {
        format!("{cluster_name}_{node_id}")
    }
}

pub fn load_cache(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<(), MetaServiceError> {
    // placement

    // journal
    let shard_storage = ShardStorage::new(rocksdb_engine_handler.clone());
    let res = shard_storage.all_shard()?;
    for shard in res {
        cache_manager.set_shard(&shard);
    }

    let segment_storage = SegmentStorage::new(rocksdb_engine_handler.clone());
    let res = segment_storage.all_segment()?;
    for segment in res {
        cache_manager.set_segment(&segment);
    }

    let segment_metadata_storage = SegmentMetadataStorage::new(rocksdb_engine_handler.clone());
    let res = segment_metadata_storage.all_segment()?;
    for meta in res {
        cache_manager.set_segment_meta(&meta);
    }

    // mqtt
    for cluster in cache_manager.get_all_cluster() {
        // Topic
        let topic = MqttTopicStorage::new(rocksdb_engine_handler.clone());
        let data = topic.list(&cluster.cluster_name)?;
        for topic in data {
            cache_manager.add_topic(&cluster.cluster_name, topic);
        }

        // User
        let user = MqttUserStorage::new(rocksdb_engine_handler.clone());
        let data = user.list_by_cluster(&cluster.cluster_name)?;
        for user in data {
            cache_manager.add_user(&cluster.cluster_name, user);
        }

        // connector
        let connector = MqttConnectorStorage::new(rocksdb_engine_handler.clone());
        let data = connector.list(&cluster.cluster_name)?;
        for connector in data {
            cache_manager.add_connector(&cluster.cluster_name, &connector);
        }
    }
    Ok(())
}
