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

use super::core::BridgePluginThread;
use common_base::tools::now_second;
use common_metrics::mqtt::connector::set_connector_up;
use dashmap::DashMap;
use metadata_struct::connector::MQTTConnector;

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
        self.connector_list
            .iter()
            .map(|connector| connector.clone())
            .collect()
    }

    pub fn remove_connector(&self, connector_name: &str) {
        self.connector_list.remove(connector_name);
    }

    // Connector Thread
    pub fn add_connector_thread(&self, connector_name: &str, thread: BridgePluginThread) {
        self.connector_thread
            .insert(connector_name.to_owned(), thread);
        let connector_type = self
            .connector_list
            .get(connector_name)
            .map(|connector| connector.connector_type.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        set_connector_up(connector_type, connector_name.to_owned(), true);
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
        let connector_type = self
            .connector_list
            .get(connector_name)
            .map(|connector| connector.connector_type.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        set_connector_up(connector_type, connector_name.to_owned(), false);
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
    use metadata_struct::connector::{
        config_local_file::LocalFileConnectorConfig, status::MQTTStatus, ConnectorType,
        FailureHandlingStrategy,
    };
    use tokio::sync::mpsc;

    fn create_test_connector(name: &str) -> MQTTConnector {
        MQTTConnector {
            connector_name: name.to_string(),
            connector_type: ConnectorType::LocalFile(LocalFileConnectorConfig::default()),
            topic_name: "test_topic".to_string(),
            failure_strategy: FailureHandlingStrategy::Discard,
            status: MQTTStatus::Running,
            broker_id: Some(1),
            create_time: now_second(),
            update_time: now_second(),
            rules: Vec::new(),
        }
    }

    fn create_test_thread() -> BridgePluginThread {
        let (stop_send, _) = mpsc::channel::<bool>(1);
        BridgePluginThread {
            connector_name: "test_connector".to_string(),
            last_send_time: 0,
            send_fail_total: 0,
            send_success_total: 0,
            stop_send,
            last_msg: None,
        }
    }

    #[test]
    fn test_connector_manager() {
        let manager = ConnectorManager::new();

        // connector CRUD
        manager.add_connector(&create_test_connector("c1"));
        manager.add_connector(&create_test_connector("c2"));
        assert_eq!(manager.get_all_connector().len(), 2);
        assert_eq!(manager.get_connector("c1").unwrap().connector_name, "c1");
        assert!(manager.get_connector("non_existent").is_none());
        manager.remove_connector("c1");
        assert_eq!(manager.get_all_connector().len(), 1);

        // thread CRUD
        manager.add_connector_thread("t1", create_test_thread());
        assert!(manager.get_connector_thread("t1").is_some());
        assert!(manager.get_connector_thread("non_existent").is_none());
        manager.remove_connector_thread("t1");
        assert!(manager.get_all_connector_thread().is_empty());

        // heartbeat
        manager.report_heartbeat("c2");
        let ts = *manager.connector_heartbeat.get("c2").unwrap();
        assert!(ts <= now_second() && ts > now_second() - 10);
    }
}
