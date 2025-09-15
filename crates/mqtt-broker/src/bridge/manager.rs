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
use metadata_struct::mqtt::bridge::connector::MQTTConnector;

use super::core::BridgePluginThread;

#[derive(Default)]
pub struct ConnectorManager {
    // (connector_name, Connector)
    pub connector_list: DashMap<String, MQTTConnector>,

    // (connector_name, BridgePluginThread)
    pub connector_thread: DashMap<String, BridgePluginThread>,

    // (connector_name, u64)
    pub connector_heartbeat: DashMap<String, u64>,
}

impl ConnectorManager {
    pub fn new() -> Self {
        ConnectorManager {
            connector_list: DashMap::with_capacity(8),
            connector_thread: DashMap::with_capacity(8),
            connector_heartbeat: DashMap::with_capacity(8),
        }
    }

    // Connector
    pub fn add_connector(&self, connector: &MQTTConnector) {
        self.connector_list
            .insert(connector.connector_name.clone(), connector.clone());
    }

    pub fn get_connector(&self, connector_name: &str) -> Option<MQTTConnector> {
        if let Some(thread) = self.connector_list.get(connector_name) {
            return Some(thread.clone());
        }

        None
    }

    pub fn get_all_connector(&self) -> Vec<MQTTConnector> {
        let mut results = Vec::new();
        for (_, raw) in self.connector_list.clone() {
            results.push(raw);
        }
        results
    }

    pub fn remove_connector(&self, connector_name: &str) {
        self.connector_list.remove(connector_name);
    }

    // Connector Thread
    pub fn add_connector_thread(&self, connector_name: &str, thread: BridgePluginThread) {
        self.connector_thread
            .insert(connector_name.to_owned(), thread);
    }

    pub fn get_connector_thread(&self, connector_name: &str) -> Option<BridgePluginThread> {
        if let Some(thread) = self.connector_thread.get(connector_name) {
            return Some(thread.clone());
        }

        None
    }

    pub fn get_all_connector_thread(&self) -> Vec<BridgePluginThread> {
        let mut results = Vec::new();
        for (_, raw) in self.connector_thread.clone() {
            results.push(raw);
        }
        results
    }

    pub fn remove_connector_thread(&self, connector_name: &str) {
        self.connector_thread.remove(connector_name);
    }

    // Connector Heartbeat
    pub fn report_heartbeat(&self, connector_name: &str) {
        self.connector_heartbeat
            .insert(connector_name.to_owned(), now_second());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::mqtt::bridge::{connector_type::ConnectorType, status::MQTTStatus};
    use tokio::sync::broadcast;

    fn create_test_connector() -> MQTTConnector {
        MQTTConnector {
            connector_name: "test_connector".to_string(),
            connector_type: ConnectorType::LocalFile,
            topic_id: "test_topic".to_string(),
            config: "{}".to_string(),
            status: MQTTStatus::Running,
            broker_id: Some(1),
            cluster_name: "test_cluster".to_string(),
            create_time: now_second(),
            update_time: now_second(),
        }
    }

    fn create_test_thread() -> BridgePluginThread {
        let (stop_send, _) = broadcast::channel::<bool>(1);
        BridgePluginThread {
            connector_name: "test_connector".to_string(),
            stop_send,
        }
    }

    #[test]
    fn connector_operations() {
        let manager = ConnectorManager::new();
        let mut connector1 = create_test_connector();
        connector1.connector_name = "connector1".to_string();
        let mut connector2 = create_test_connector();
        connector2.connector_name = "connector2".to_string();

        // add
        manager.add_connector(&connector1);

        // get
        let retrieved = manager.get_connector("connector1");
        assert!(retrieved.is_some());
        let retrieved_connector = retrieved.unwrap();
        assert_eq!(retrieved_connector.connector_name, "connector1");
        assert_eq!(retrieved_connector.topic_id, "test_topic");
        assert!(manager.get_connector("non_existent").is_none());

        // remove
        manager.remove_connector("connector1");
        assert!(manager.get_all_connector().is_empty());

        // add again
        manager.add_connector(&connector1);
        manager.add_connector(&connector2);

        // get all connectors
        assert_eq!(manager.get_all_connector().len(), 2);
    }

    #[test]
    fn connector_thread_operations() {
        let manager = ConnectorManager::new();
        let thread1 = create_test_thread();
        let thread2 = create_test_thread();

        // add
        manager.add_connector_thread("connector1", thread1);

        // get
        let retrieved = manager.get_connector_thread("connector1");
        assert_eq!(retrieved.unwrap().connector_name, "test_connector");
        assert!(manager.get_connector_thread("non_existent").is_none());

        // remove
        manager.remove_connector_thread("connector1");
        assert!(manager.get_all_connector_thread().is_empty());

        // add again
        manager.add_connector_thread("connector2", thread2);

        // get all connectors
        let all = manager.get_all_connector_thread();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn connector_heartbeat_operations() {
        let manager = ConnectorManager::new();

        manager.report_heartbeat("test_connector");

        assert!(manager.connector_heartbeat.contains_key("test_connector"));
        let heartbeat_time = manager.connector_heartbeat.get("test_connector").unwrap();

        let current_time = now_second();
        assert!(heartbeat_time.value() <= &current_time);
        assert!(heartbeat_time.value() > &(current_time - 10));
    }
}
