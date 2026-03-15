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
    // (tenant, (connector_name, Connector))
    pub connector_list: DashMap<String, DashMap<String, MQTTConnector>>,

    // (tenant, (connector_name, BridgePluginThread))
    pub connector_thread: DashMap<String, DashMap<String, BridgePluginThread>>,

    // (tenant, (connector_name, u64))
    pub connector_heartbeat: DashMap<String, DashMap<String, u64>>,
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
            .entry(connector.tenant.clone())
            .or_default()
            .insert(connector.connector_name.clone(), connector.clone());
    }

    pub fn get_connector(&self, connector_name: &str) -> Option<MQTTConnector> {
        for tenant_entry in self.connector_list.iter() {
            if let Some(conn) = tenant_entry.value().get(connector_name) {
                return Some(conn.clone());
            }
        }
        None
    }

    pub fn get_connector_by_tenant(
        &self,
        tenant: &str,
        connector_name: &str,
    ) -> Option<MQTTConnector> {
        self.connector_list
            .get(tenant)
            .and_then(|m| m.get(connector_name).map(|c| c.clone()))
    }

    pub fn get_all_connector(&self) -> Vec<MQTTConnector> {
        self.connector_list
            .iter()
            .flat_map(|tenant_entry| {
                tenant_entry
                    .value()
                    .iter()
                    .map(|e| e.value().clone())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn get_connector_by_tenant_list(&self, tenant: &str) -> Vec<MQTTConnector> {
        self.connector_list
            .get(tenant)
            .map(|m| m.iter().map(|e| e.value().clone()).collect())
            .unwrap_or_default()
    }

    pub fn remove_connector(&self, connector_name: &str) {
        for tenant_entry in self.connector_list.iter() {
            tenant_entry.value().remove(connector_name);
        }
    }

    // Connector Thread
    pub fn add_connector_thread(
        &self,
        tenant: &str,
        connector_name: &str,
        thread: BridgePluginThread,
    ) {
        let connector_type = self
            .get_connector_by_tenant(tenant, connector_name)
            .map(|c| c.connector_type.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        self.connector_thread
            .entry(tenant.to_owned())
            .or_default()
            .insert(connector_name.to_owned(), thread);
        set_connector_up(tenant, connector_type, connector_name.to_owned(), true);
    }

    pub fn get_connector_thread(&self, connector_name: &str) -> Option<BridgePluginThread> {
        for tenant_entry in self.connector_thread.iter() {
            if let Some(thread) = tenant_entry.value().get(connector_name) {
                return Some(thread.clone());
            }
        }
        None
    }

    pub fn get_all_connector_thread(&self) -> Vec<BridgePluginThread> {
        self.connector_thread
            .iter()
            .flat_map(|tenant_entry| {
                tenant_entry
                    .value()
                    .iter()
                    .map(|e| e.value().clone())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn remove_connector_thread(&self, connector_name: &str) {
        let connector = self.get_connector(connector_name);
        let connector_type = connector
            .as_ref()
            .map(|c| c.connector_type.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let tenant = connector.as_ref().map(|c| c.tenant.as_str()).unwrap_or("");
        set_connector_up(tenant, connector_type, connector_name.to_owned(), false);
        for tenant_entry in self.connector_thread.iter() {
            tenant_entry.value().remove(connector_name);
        }
    }

    pub fn update_connector_thread_last_active(
        &self,
        connector_name: &str,
        f: impl Fn(&mut BridgePluginThread),
    ) {
        for tenant_entry in self.connector_thread.iter() {
            if let Some(mut thread) = tenant_entry.value().get_mut(connector_name) {
                f(&mut thread);
                return;
            }
        }
    }

    // Connector Heartbeat
    pub fn report_heartbeat(&self, tenant: &str, connector_name: &str) {
        self.connector_heartbeat
            .entry(tenant.to_owned())
            .or_default()
            .insert(connector_name.to_owned(), now_second());
    }

    pub fn get_all_heartbeats(&self) -> Vec<(String, u64)> {
        self.connector_heartbeat
            .iter()
            .flat_map(|tenant_entry| {
                tenant_entry
                    .value()
                    .iter()
                    .map(|e| (e.key().clone(), *e.value()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn connector_count(&self) -> usize {
        self.connector_list.iter().map(|e| e.value().len()).sum()
    }

    pub fn connector_thread_count(&self) -> usize {
        self.connector_thread.iter().map(|e| e.value().len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::{
        connector::{
            config_local_file::LocalFileConnectorConfig, rule::ETLRule, status::MQTTStatus,
            ConnectorType, FailureHandlingStrategy,
        },
        tenant::DEFAULT_TENANT,
    };
    use tokio::sync::mpsc;

    fn create_test_connector(name: &str) -> MQTTConnector {
        MQTTConnector {
            connector_name: name.to_string(),
            connector_type: ConnectorType::LocalFile(LocalFileConnectorConfig::default()),
            tenant: DEFAULT_TENANT.to_string(),
            topic_name: "test_topic".to_string(),
            failure_strategy: FailureHandlingStrategy::Discard,
            status: MQTTStatus::Running,
            broker_id: Some(1),
            create_time: now_second(),
            update_time: now_second(),
            etl_rule: ETLRule::default(),
        }
    }

    fn create_test_thread(name: &str) -> BridgePluginThread {
        let (stop_send, _) = mpsc::channel::<bool>(1);
        BridgePluginThread {
            connector_name: name.to_string(),
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
        manager.add_connector(&create_test_connector("t1"));
        manager.add_connector_thread(DEFAULT_TENANT, "t1", create_test_thread("t1"));
        assert!(manager.get_connector_thread("t1").is_some());
        assert!(manager.get_connector_thread("non_existent").is_none());
        manager.remove_connector_thread("t1");
        assert!(manager.get_all_connector_thread().is_empty());

        // heartbeat
        manager.report_heartbeat(DEFAULT_TENANT, "c2");
        let ts = manager
            .connector_heartbeat
            .get(DEFAULT_TENANT)
            .and_then(|m| m.get("c2").map(|v| *v))
            .unwrap();
        assert!(ts <= now_second() && ts > now_second() - 10);
    }
}
