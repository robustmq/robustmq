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

use common_base::tools::now_second;
use dashmap::DashMap;
use log::error;
use metadata_struct::placement::{broker_node::BrokerNode, cluster::ClusterInfo};
use raft::StateRole;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    core::{cluster::ClusterMetadata, raft_node::RaftNode},
    storage::{
        placement::{cluster::ClusterStorage, node::NodeStorage},
        rocksdb::RocksDBEngine,
    },
};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PlacementCacheManager {
    // placement raft cluster
    pub placement_cluster: DashMap<String, ClusterMetadata>,

    // broker cluster & node
    pub cluster_list: DashMap<String, ClusterInfo>,
    pub node_list: DashMap<String, DashMap<u64, BrokerNode>>,
    pub node_heartbeat: DashMap<String, DashMap<u64, u64>>,
}

impl PlacementCacheManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> PlacementCacheManager {
        let mut cache = PlacementCacheManager {
            cluster_list: DashMap::with_capacity(2),
            node_heartbeat: DashMap::with_capacity(2),
            node_list: DashMap::with_capacity(2),
            placement_cluster: DashMap::with_capacity(2),
        };
        cache.load_cache(rocksdb_engine_handler);
        return cache;
    }

    pub fn add_broker_cluster(&self, cluster: &ClusterInfo) {
        self.cluster_list
            .insert(cluster.cluster_name.clone(), cluster.clone());
    }

    pub fn add_broker_node(&self, node: BrokerNode) {
        self.report_heart_by_broker_node(&node.cluster_name, node.node_id, now_second());

        if let Some(data) = self.node_list.get_mut(&node.cluster_name) {
            data.insert(node.node_id, node);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(node.node_id, node.clone());
            self.node_list.insert(node.cluster_name.clone(), data);
        }
    }

    pub fn remove_broker_node(&self, cluster_name: &String, node_id: u64) {
        if let Some(data) = self.node_list.get_mut(cluster_name) {
            data.remove(&node_id);
        }
        if let Some(data) = self.node_heartbeat.get_mut(cluster_name) {
            data.remove(&node_id);
        }
    }

    pub fn get_broker_node(&self, cluster_name: &String, node_id: u64) -> Option<BrokerNode> {
        if let Some(data) = self.node_list.get_mut(cluster_name) {
            if let Some(value) = data.get(&node_id) {
                return Some(value.clone());
            }
        }
        return None;
    }

    pub fn get_broker_node_addr_by_cluster(&self, cluster_name: &String) -> Vec<String> {
        let mut results = Vec::new();
        if let Some(data) = self.node_list.get_mut(cluster_name) {
            for (_, node) in data.clone() {
                if node.cluster_name.eq(cluster_name) {
                    results.push(node.node_inner_addr);
                }
            }
        }
        return results;
    }

    pub fn report_heart_by_broker_node(&self, cluster_name: &String, node_id: u64, time: u64) {
        if let Some(data) = self.node_heartbeat.get_mut(cluster_name) {
            data.insert(node_id, time);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(node_id, time);
            self.node_heartbeat.insert(cluster_name.clone(), data);
        }
    }

    pub fn load_cache(&mut self, rocksdb_engine_handler: Arc<RocksDBEngine>) {
        let cluster = ClusterStorage::new(rocksdb_engine_handler.clone());
        match cluster.list(None) {
            Ok(result) => {
                for cluster in result {
                    self.add_broker_cluster(&cluster);
                }
            }
            Err(_) => {}
        }

        let node = NodeStorage::new(rocksdb_engine_handler.clone());
        match node.list(None) {
            Ok(result) => {
                for bn in result {
                    self.add_broker_node(bn);
                }
            }
            Err(_) => {}
        }

        let placement_cluster = DashMap::with_capacity(2);
        placement_cluster.insert(self.cluster_key(), ClusterMetadata::new());
        self.placement_cluster = placement_cluster;
    }

    pub fn add_raft_memner(&self, node: RaftNode) {
        if let Some(mut cluster) = self.placement_cluster.get_mut(&self.cluster_key()) {
            cluster.add_member(node.node_id, node);
        }
    }

    pub fn remove_raft_memner(&self, id: u64) {
        if let Some(mut cluster) = self.placement_cluster.get_mut(&self.cluster_key()) {
            cluster.remove_member(id);
        }
    }

    pub fn get_raft_votes(&self) -> Vec<RaftNode> {
        if let Some(cluster) = self.placement_cluster.get(&self.cluster_key()) {
            return cluster.votes.iter().map(|v| v.clone()).collect();
        }
        return Vec::new();
    }

    pub fn get_raft_members(&self) -> Vec<RaftNode> {
        if let Some(cluster) = self.placement_cluster.get(&self.cluster_key()) {
            return cluster.members.iter().map(|v| v.clone()).collect();
        }
        return Vec::new();
    }

    pub fn get_votes_node_by_id(&self, node_id: u64) -> Option<RaftNode> {
        if let Some(cluster) = self.placement_cluster.get(&self.cluster_key()) {
            if let Some(node) = cluster.get_node_by_id(node_id) {
                return Some(node.clone());
            }
        }
        return None;
    }

    pub fn is_raft_role_change(&self, new_role: StateRole) -> bool {
        if let Some(cluster) = self.placement_cluster.get(&self.cluster_key()) {
            return cluster.is_raft_role_change(new_role);
        }
        return false;
    }

    pub fn is_leader(&self) -> bool {
        if let Some(cluster) = self.placement_cluster.get(&self.cluster_key()) {
            return cluster.is_leader();
        }
        return false;
    }

    pub fn get_raft_leader(&self) -> Option<RaftNode> {
        if let Some(cluster) = self.placement_cluster.get(&self.cluster_key()) {
            return cluster.leader.clone();
        }
        return None;
    }

    pub fn get_raft_local_node(&self) -> Option<RaftNode> {
        if let Some(cluster) = self.placement_cluster.get(&self.cluster_key()) {
            return Some(cluster.local.clone());
        }
        return None;
    }

    pub fn get_current_raft_role(&self) -> String {
        if let Some(cluster) = self.placement_cluster.get(&self.cluster_key()) {
            return cluster.raft_role.clone();
        }
        return "".to_string();
    }

    pub fn update_raft_role(&self, local_new_role: StateRole, leader_id: u64) {
        if let Some(mut cluster) = self.placement_cluster.get_mut(&self.cluster_key()) {
            cluster.update_node_raft_role(local_new_role);
            
            if leader_id > 0 {
                if let Some(leader) = cluster.votes.clone().get(&leader_id) {
                    cluster.set_leader(Some(leader.clone()));
                } else {
                    error!("Invalid leader id, the node corresponding to the leader id cannot be found in votes.");
                }
            }
        }
    }

    fn cluster_key(&self) -> String {
        return "cluster".to_string();
    }
}
