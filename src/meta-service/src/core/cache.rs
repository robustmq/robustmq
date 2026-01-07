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
use crate::storage::common::node::NodeStorage;
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use crate::storage::journal::shard::ShardStorage;
use crate::storage::mqtt::connector::MqttConnectorStorage;
use crate::storage::mqtt::user::MqttUserStorage;
use crate::{controller::session_expire::ExpireLastWill, storage::mqtt::topic::MqttTopicStorage};
use common_base::role::is_engine_node;
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::topic::Topic;
use metadata_struct::mqtt::user::MqttUser;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct CacheManager {
    // (node_id, BrokerNode)
    pub node_list: DashMap<u64, BrokerNode>,

    // (node_id, NodeHeartbeatData)
    pub node_heartbeat: DashMap<u64, NodeHeartbeatData>,

    // MQTT
    // (username,(topic_name,topic))
    pub topic_list: DashMap<String, Topic>,

    // (username,user)
    pub user_list: DashMap<String, MqttUser>,

    // (client_id, ExpireLastWill)
    pub expire_last_wills: DashMap<String, ExpireLastWill>,

    // (client_id,MQTTConnector)
    pub connector_list: DashMap<String, MQTTConnector>,

    //(connector_name, ConnectorHeartbeat)
    pub connector_heartbeat: DashMap<String, ConnectorHeartbeat>,

    // Storage Engine
    //（shard_name, JournalShard）
    pub shard_list: DashMap<String, EngineShard>,

    //（shard_name, (segment_no,JournalSegment))
    pub segment_list: DashMap<String, DashMap<u32, EngineSegment>>,

    //（shard_name, (segment_no,JournalSegmentMetadata))
    pub segment_meta_list: DashMap<String, DashMap<u32, EngineSegmentMetadata>>,

    //（shard_name, delete_time）
    pub wait_delete_shard_list: DashMap<String, u64>,

    //（shard_name, JournalSegment)
    pub wait_delete_segment_list: DashMap<String, EngineSegment>,
}

impl CacheManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> CacheManager {
        let mut cache = CacheManager {
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

    // Node
    pub fn add_broker_node(&self, node: BrokerNode) {
        self.node_list.insert(node.node_id, node);
    }

    pub fn remove_broker_node(&self, node_id: u64) -> Option<(u64, BrokerNode)> {
        self.node_list.remove(&node_id);
        self.node_heartbeat.remove(&node_id);
        None
    }

    pub fn get_broker_node(&self, node_id: u64) -> Option<BrokerNode> {
        if let Some(data) = self.node_list.get(&node_id) {
            return Some(data.clone());
        }
        None
    }

    pub fn get_engine_node_list(&self) -> Vec<BrokerNode> {
        let mut results = Vec::new();
        println!("nl:{:?}", self.node_list);
        for node in self.node_list.iter() {
            if is_engine_node(&node.roles) {
                results.push(node.clone());
            }
        }
        println!("results:{:?}", results);
        results
    }

    // Heartbeat
    pub fn report_broker_heart(&self, node_id: u64) {
        let data = NodeHeartbeatData {
            node_id,
            time: now_second(),
        };
        self.node_heartbeat.insert(node_id, data);
    }

    pub fn get_broker_heart(&self, node_id: u64) -> Option<NodeHeartbeatData> {
        if let Some(heart) = self.node_heartbeat.get(&node_id) {
            return Some(heart.clone());
        }
        None
    }

    pub fn load_cache(&mut self, rocksdb_engine_handler: Arc<RocksDBEngine>) {
        let node = NodeStorage::new(rocksdb_engine_handler);
        if let Ok(result) = node.list() {
            for bn in result {
                self.add_broker_node(bn);
            }
        }
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
        cache_manager.set_shard(shard);
    }

    let segment_storage = SegmentStorage::new(rocksdb_engine_handler.clone());
    let res = segment_storage.all_segment()?;
    for segment in res {
        cache_manager.set_segment(segment);
    }

    let segment_metadata_storage = SegmentMetadataStorage::new(rocksdb_engine_handler.clone());
    let res = segment_metadata_storage.all_segment()?;
    for meta in res {
        cache_manager.set_segment_meta(meta);
    }

    // Topic
    let topic = MqttTopicStorage::new(rocksdb_engine_handler.clone());
    let data = topic.list()?;
    for topic in data {
        cache_manager.add_topic(topic);
    }

    // User
    let user = MqttUserStorage::new(rocksdb_engine_handler.clone());
    let data = user.list()?;
    for user in data {
        cache_manager.add_user(user);
    }

    // connector
    let connector = MqttConnectorStorage::new(rocksdb_engine_handler.clone());
    let data = connector.list()?;
    for connector in data {
        cache_manager.add_connector(connector);
    }

    Ok(())
}
