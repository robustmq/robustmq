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

use arc_swap::ArcSwap;
use common_base::{node_status::NodeStatus, tools::now_second};
use common_config::config::BrokerConfig;
use dashmap::{DashMap, DashSet};
use metadata_struct::{
    meta::node::BrokerNode,
    mqtt::{session::MqttSession, share_group::ShareGroupLeader, topic::Topic},
    tenant::Tenant,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct NodeCacheManager {
    // start_time
    pub start_time: u64,

    // (tenant_name, Tenant)
    pub tenant_list: DashMap<String, Tenant>,

    // node list
    pub node_lists: DashMap<u64, BrokerNode>,

    // cluster_name
    pub cluster_name: String,

    // (cluster_name, Cluster)
    pub cluster_config: ArcSwap<BrokerConfig>,

    // ("{tenant}/{topic_name}", Topic)
    pub topic_list: DashMap<String, Topic>,
    // tenant -> {"{tenant}/{topic_name}"}
    pub topic_tenant_index: DashMap<String, DashSet<String>>,

    // ("{tenant}/{group_name}", ShareGroupLeader)
    pub share_group_list: DashMap<String, ShareGroupLeader>,

    // ("{tenant}/{client_id}", MqttSession)
    pub session_list: DashMap<String, MqttSession>,
    // tenant -> {"{tenant}/{client_id}"}
    pub session_tenant_index: DashMap<String, DashSet<String>>,

    // (cluster_name, Status)
    pub status: Arc<RwLock<NodeStatus>>,
}
impl NodeCacheManager {
    pub fn new(cluster: BrokerConfig) -> Self {
        NodeCacheManager {
            cluster_name: cluster.cluster_name.clone(),
            start_time: now_second(),
            tenant_list: DashMap::with_capacity(8),
            node_lists: DashMap::with_capacity(2),
            cluster_config: ArcSwap::new(Arc::new(cluster)),
            status: Arc::new(RwLock::new(NodeStatus::Starting)),
            share_group_list: DashMap::new(),
            session_list: DashMap::new(),
            session_tenant_index: DashMap::with_capacity(8),
            topic_list: DashMap::new(),
            topic_tenant_index: DashMap::with_capacity(8),
        }
    }

    // Tenant
    pub fn add_tenant(&self, tenant: Tenant) {
        self.tenant_list.insert(tenant.tenant_name.clone(), tenant);
    }

    pub fn remove_tenant(&self, tenant_name: &str) {
        self.tenant_list.remove(tenant_name);
    }

    pub fn tenant_exists(&self, tenant_name: &str) -> bool {
        self.tenant_list.contains_key(tenant_name)
    }

    pub fn get_tenant(&self, tenant_name: &str) -> Option<Tenant> {
        self.tenant_list.get(tenant_name).map(|t| t.clone())
    }

    // node
    pub fn add_node(&self, node: BrokerNode) {
        self.node_lists.insert(node.node_id, node);
    }

    pub fn remove_node(&self, node: BrokerNode) {
        self.node_lists.remove(&node.node_id);
    }

    pub fn node_list(&self) -> Vec<BrokerNode> {
        self.node_lists
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    // Session
    pub fn add_session(&self, session: MqttSession) {
        let key = format!("{}/{}", session.tenant, session.client_id);
        self.session_tenant_index
            .entry(session.tenant.clone())
            .or_default()
            .insert(key.clone());
        self.session_list.insert(key, session);
    }

    pub fn delete_session(&self, tenant: &str, client_id: &str) {
        let key = format!("{tenant}/{client_id}");
        self.session_list.remove(&key);
        if let Some(set) = self.session_tenant_index.get(tenant) {
            set.remove(&key);
        }
    }

    pub fn get_session(&self, tenant: &str, client_id: &str) -> Option<MqttSession> {
        let key = format!("{tenant}/{client_id}");
        self.session_list.get(&key).map(|s| s.clone())
    }

    pub fn list_sessions_by_tenant(&self, tenant: &str) -> Vec<MqttSession> {
        self.session_tenant_index
            .get(tenant)
            .map(|keys| {
                keys.iter()
                    .filter_map(|k| self.session_list.get(k.as_str()).map(|s| s.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    // Topic
    pub fn add_topic(&self, topic: &Topic) {
        let key = format!("{}/{}", topic.tenant, topic.topic_name);
        self.topic_tenant_index
            .entry(topic.tenant.clone())
            .or_default()
            .insert(key.clone());
        self.topic_list.insert(key, topic.clone());
    }

    pub fn delete_topic(&self, tenant: &str, topic_name: &str) {
        let key = format!("{tenant}/{topic_name}");
        self.topic_list.remove(&key);
        if let Some(set) = self.topic_tenant_index.get(tenant) {
            set.remove(&key);
        }
    }

    pub fn topic_exists(&self, tenant: &str, topic_name: &str) -> bool {
        let key = format!("{tenant}/{topic_name}");
        self.topic_list.contains_key(&key)
    }

    pub fn get_topic_by_name(&self, tenant: &str, topic_name: &str) -> Option<Topic> {
        let key = format!("{tenant}/{topic_name}");
        self.topic_list.get(&key).map(|t| t.clone())
    }

    pub fn list_topics_by_tenant(&self, tenant: &str) -> Vec<Topic> {
        self.topic_tenant_index
            .get(tenant)
            .map(|keys| {
                keys.iter()
                    .filter_map(|k| self.topic_list.get(k.as_str()).map(|t| t.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_all_topic_name(&self) -> Vec<String> {
        self.topic_list
            .iter()
            .map(|e| e.value().topic_name.clone())
            .collect()
    }

    pub fn topic_count(&self) -> usize {
        self.topic_list.len()
    }

    pub fn topic_count_by_tenant(&self, tenant: &str) -> usize {
        self.topic_tenant_index
            .get(tenant)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    // ShareGroup
    pub fn add_share_group(&self, group: ShareGroupLeader) {
        let key = format!("{}/{}", group.tenant, group.group_name);
        self.share_group_list.insert(key, group);
    }

    pub fn remove_share_group(&self, tenant: &str, group_name: &str) {
        let key = format!("{tenant}/{group_name}");
        self.share_group_list.remove(&key);
    }

    pub fn get_share_group(&self, tenant: &str, group_name: &str) -> Option<ShareGroupLeader> {
        let key = format!("{tenant}/{group_name}");
        self.share_group_list.get(&key).map(|g| g.clone())
    }

    // get start time
    pub fn get_start_time(&self) -> u64 {
        self.start_time
    }

    // status
    pub async fn set_status(&self, status: NodeStatus) {
        let mut data = self.status.write().await;
        *data = status;
    }

    pub async fn get_status(&self) -> NodeStatus {
        self.status.read().await.clone()
    }

    pub async fn is_stop(&self) -> bool {
        self.get_status().await == NodeStatus::Stopping
    }

    // cluster config
    pub fn set_cluster_config(&self, config: BrokerConfig) {
        self.cluster_config.store(Arc::new(config));
    }

    pub fn get_cluster_config(&self) -> BrokerConfig {
        self.cluster_config.load().as_ref().clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::NodeCacheManager;
    use common_base::tools::now_second;
    use common_config::broker::default_broker_config;
    use metadata_struct::meta::node::BrokerNode;

    #[tokio::test]
    async fn start_time_operations() {
        let cache_manager = NodeCacheManager::new(default_broker_config());
        let start_time = cache_manager.get_start_time();
        assert!(start_time > 0);
        assert!(start_time <= now_second());
    }

    #[tokio::test]
    async fn node_operations() {
        let cache_manager = NodeCacheManager::new(default_broker_config());
        let node = BrokerNode {
            node_id: 1,
            node_ip: "127.0.0.1".to_string(),
            ..Default::default()
        };

        // add
        cache_manager.add_node(node.clone());

        // get
        let nodes = cache_manager.node_list();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_id, node.node_id);
        assert_eq!(nodes[0].node_ip, node.node_ip);

        // remove
        cache_manager.remove_node(node.clone());

        // get again
        let nodes = cache_manager.node_list();
        assert!(nodes.is_empty());
    }
}
