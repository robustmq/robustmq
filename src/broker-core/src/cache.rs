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

use common_base::{node_status::NodeStatus, tools::now_second};
use common_config::config::BrokerConfig;
use dashmap::DashMap;
use metadata_struct::{
    meta::node::BrokerNode,
    mqtt::{session::MqttSession, topic::Topic},
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
    pub cluster_config: Arc<RwLock<BrokerConfig>>,

    // (tenant, (topic_name, Topic))
    pub topic_list: DashMap<String, DashMap<String, Topic>>,

    // (tenant, (client_id, MqttSession))
    pub session_list: DashMap<String, DashMap<String, MqttSession>>,

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
            cluster_config: Arc::new(RwLock::new(cluster)),
            status: Arc::new(RwLock::new(NodeStatus::Starting)),
            session_list: DashMap::with_capacity(8),
            topic_list: DashMap::with_capacity(2),
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
        self.session_list
            .entry(session.tenant.clone())
            .or_default()
            .insert(session.client_id.clone(), session);
    }

    pub fn delete_session(&self, tenant: &str, client_id: &str) {
        if let Some(tenant_map) = self.session_list.get(tenant) {
            tenant_map.remove(client_id);
        }
    }

    pub fn get_session(&self, tenant: &str, client_id: &str) -> Option<MqttSession> {
        self.session_list
            .get(tenant)?
            .get(client_id)
            .map(|s| s.clone())
    }

    pub fn list_sessions_by_tenant(&self, tenant: &str) -> Vec<MqttSession> {
        self.session_list
            .get(tenant)
            .map(|tenant_map| tenant_map.iter().map(|e| e.value().clone()).collect())
            .unwrap_or_default()
    }

    // Topic
    pub fn add_topic(&self, topic: &Topic) {
        self.topic_list
            .entry(topic.tenant.clone())
            .or_default()
            .insert(topic.topic_name.clone(), topic.clone());
    }

    pub fn delete_topic(&self, tenant: &str, topic_name: &str) {
        if let Some(tenant_map) = self.topic_list.get(tenant) {
            tenant_map.remove(topic_name);
        }
    }

    pub fn topic_exists(&self, tenant: &str, topic_name: &str) -> bool {
        self.topic_list
            .get(tenant)
            .map(|m| m.contains_key(topic_name))
            .unwrap_or(false)
    }

    pub fn get_topic_by_name(&self, tenant: &str, topic_name: &str) -> Option<Topic> {
        self.topic_list
            .get(tenant)?
            .get(topic_name)
            .map(|t| t.clone())
    }

    pub fn list_topics_by_tenant(&self, tenant: &str) -> Vec<Topic> {
        self.topic_list
            .get(tenant)
            .map(|m| m.iter().map(|e| e.value().clone()).collect())
            .unwrap_or_default()
    }

    pub fn get_all_topic_name(&self) -> Vec<String> {
        self.topic_list
            .iter()
            .flat_map(|entry| {
                entry
                    .value()
                    .iter()
                    .map(|t| t.topic_name.clone())
                    .collect::<Vec<_>>()
            })
            .collect()
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
    pub async fn set_cluster_config(&self, config: BrokerConfig) {
        let mut data = self.cluster_config.write().await;
        *data = config;
    }

    pub async fn get_cluster_config(&self) -> BrokerConfig {
        self.cluster_config.read().await.clone()
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
