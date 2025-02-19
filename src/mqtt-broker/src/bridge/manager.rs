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

use dashmap::DashMap;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;

use super::core::BridgePluginThread;

#[derive(Default)]
pub struct ConnectorManager {
    // (connector_name, Connector)
    pub connector_list: DashMap<String, MQTTConnector>,

    // (connector_name, BridgePluginThread)
    pub connector_thread: DashMap<String, BridgePluginThread>,
}

impl ConnectorManager {
    pub fn new() -> Self {
        ConnectorManager {
            connector_list: DashMap::with_capacity(8),
            connector_thread: DashMap::with_capacity(8),
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
}
