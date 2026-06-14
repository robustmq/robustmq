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
use crate::storage::common::tenant::TenantStorage;
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use crate::storage::journal::shard::ShardStorage;
use crate::storage::mqtt::connector::MqttConnectorStorage;
use common_base::role::is_engine_node;
use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::connector::MQTTConnector;
use metadata_struct::meta::node::BrokerNode;
use metadata_struct::mqtt::share_group::ShareGroup;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use metadata_struct::tenant::Tenant;
use rocksdb_engine::rocksdb::RocksDBEngine;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Per-node replica/leader placement load, maintained incrementally by
/// `set_segment`/`remove_segment` and lazily initialized on first read.
#[derive(Clone, Default, Debug)]
pub struct NodeLoadCache {
    pub(crate) replica_count: DashMap<u64, u64>,
    pub(crate) leader_count: DashMap<u64, u64>,
    pub(crate) initialized: Arc<AtomicBool>,
    pub(crate) init_lock: Arc<Mutex<()>>,
}

impl NodeLoadCache {
    /// Add (`delta` = +1) or remove (`delta` = -1) a segment's replica/leader
    /// contribution. No-op until initialized — the first scan sets the baseline.
    pub(crate) fn apply(&self, segment: &EngineSegment, delta: i64) {
        if !self.initialized.load(Ordering::Acquire) {
            return;
        }
        for replica in &segment.replicas {
            adjust_count(&self.replica_count, replica.node_id, delta);
        }
        adjust_count(&self.leader_count, segment.leader, delta);
    }

    pub(crate) fn remove_node(&self, node_id: u64) {
        self.replica_count.remove(&node_id);
        self.leader_count.remove(&node_id);
    }
}

fn adjust_count(map: &DashMap<u64, u64>, node_id: u64, delta: i64) {
    let mut entry = map.entry(node_id).or_insert(0);
    *entry = (*entry as i64 + delta).max(0) as u64;
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct MetaCacheManager {
    // (tenant_name, Tenant)
    pub tenant_list: DashMap<String, Tenant>,

    // (node_id, BrokerNode)
    pub node_list: DashMap<u64, BrokerNode>,

    // (node_id, NodeHeartbeatData)
    pub node_heartbeat: DashMap<u64, NodeHeartbeatData>,

    // MQTT
    // (client_id, MQTTConnector)
    pub connector_list: DashMap<String, MQTTConnector>,

    //(connector_name, ConnectorHeartbeat)
    pub connector_heartbeat: DashMap<String, ConnectorHeartbeat>,

    // (group_name, broker_id)
    pub group_leader: DashMap<String, ShareGroup>,

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

    // Per-node replica/leader placement load (not persisted; rebuilt on demand).
    #[serde(skip)]
    pub node_load: NodeLoadCache,
}

impl MetaCacheManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> MetaCacheManager {
        let mut cache = MetaCacheManager {
            tenant_list: DashMap::with_capacity(8),
            node_heartbeat: DashMap::with_capacity(2),
            node_list: DashMap::with_capacity(2),
            connector_list: DashMap::with_capacity(8),
            connector_heartbeat: DashMap::with_capacity(8),
            shard_list: DashMap::with_capacity(8),
            segment_list: DashMap::with_capacity(256),
            segment_meta_list: DashMap::with_capacity(256),
            wait_delete_shard_list: DashMap::with_capacity(8),
            wait_delete_segment_list: DashMap::with_capacity(8),
            group_leader: DashMap::with_capacity(8),
            node_load: NodeLoadCache::default(),
        };
        cache.load_cache(rocksdb_engine_handler);
        cache
    }

    // Tenant
    pub fn add_tenant(&self, tenant: Tenant) {
        self.tenant_list.insert(tenant.tenant_name.clone(), tenant);
    }

    pub fn remove_tenant(&self, tenant_name: &str) {
        self.tenant_list.remove(tenant_name);
    }

    pub fn get_tenant(&self, tenant_name: &str) -> Option<Tenant> {
        self.tenant_list.get(tenant_name).map(|t| t.clone())
    }

    pub fn tenant_exists(&self, tenant_name: &str) -> bool {
        self.tenant_list.contains_key(tenant_name)
    }

    // Node
    pub fn add_broker_node(&self, node: BrokerNode) {
        self.node_list.insert(node.node_id, node);
    }

    pub fn remove_broker_node(&self, node_id: u64) -> Option<(u64, BrokerNode)> {
        self.node_list.remove(&node_id);
        self.node_heartbeat.remove(&node_id);
        self.node_load.remove_node(node_id);
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
        for node in self.node_list.iter() {
            if is_engine_node(&node.roles) {
                results.push(node.clone());
            }
        }
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

pub fn load_cache_by_rocksdb(
    cache_manager: &Arc<MetaCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<(), MetaServiceError> {
    let tenant_storage = TenantStorage::new(rocksdb_engine_handler.clone());
    for tenant in tenant_storage.list()? {
        cache_manager.add_tenant(tenant);
    }

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

    let connector = MqttConnectorStorage::new(rocksdb_engine_handler.clone());
    let data = connector.list()?;
    for connector in data {
        cache_manager.add_connector(connector);
    }

    Ok(())
}
