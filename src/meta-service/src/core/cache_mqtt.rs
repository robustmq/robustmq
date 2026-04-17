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

use crate::core::cache::MetaCacheManager;
use crate::server::services::mqtt::connector::ConnectorHeartbeat;
use metadata_struct::connector::MQTTConnector;
use metadata_struct::mqtt::share_group::ShareGroupLeader;

impl MetaCacheManager {
    pub fn add_group_leader(&self, group_info: ShareGroupLeader) {
        let key = format!("{}/{}", group_info.tenant, group_info.group_name);
        self.group_leader.insert(key, group_info);
    }

    pub fn remove_group_leader(&self, tenant: &str, group_name: &str) {
        let key = format!("{}/{}", tenant, group_name);
        self.group_leader.remove(&key);
    }

    pub fn get_group_leader(&self, tenant: &str, group_name: &str) -> Option<ShareGroupLeader> {
        let key = format!("{}/{}", tenant, group_name);
        self.group_leader.get(&key).map(|v| v.clone())
    }

    pub fn add_connector(&self, connector: MQTTConnector) {
        self.connector_list
            .insert(connector.connector_name.clone(), connector);
    }

    pub fn remove_connector(&self, connector_name: &str) {
        self.connector_list.remove(connector_name);
    }

    pub fn get_all_connector(&self) -> Vec<MQTTConnector> {
        self.connector_list
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn report_connector_heartbeat(&self, connector_name: &str, heartbeat_time: u64) {
        let name = connector_name.to_string();
        let heartbeat = ConnectorHeartbeat {
            connector_name: name.clone(),
            last_heartbeat: heartbeat_time,
        };
        self.connector_heartbeat.insert(name, heartbeat);
    }

    pub fn remove_connector_heartbeat(&self, connector_name: &str) {
        self.connector_heartbeat.remove(connector_name);
    }

    pub fn get_all_connector_heartbeat(&self) -> Vec<ConnectorHeartbeat> {
        self.connector_heartbeat
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}
