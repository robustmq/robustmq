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

use crate::controller::mqtt::is_send_last_will;
use crate::controller::mqtt::session_expire::ExpireLastWill;
use crate::core::cache::CacheManager;
use crate::server::services::mqtt::connector::ConnectorHeartbeat;
use dashmap::DashMap;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::user::MqttUser;

impl CacheManager {
    // Topic
    pub fn add_topic(&self, topic: MQTTTopic) {
        self.topic_list.insert(topic.topic_name.to_owned(), topic);
    }

    pub fn remove_topic(&self, topic_name: &str) {
        self.topic_list.remove(topic_name);
    }

    // User
    pub fn add_user(&self, cluster_name: &str, user: MqttUser) {
        if let Some(data) = self.user_list.get_mut(cluster_name) {
            data.insert(user.username.clone(), user);
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(user.username.clone(), user);
            self.user_list.insert(cluster_name.to_owned(), data);
        }
    }

    pub fn remove_user(&self, cluster_name: &str, user_name: &str) {
        if let Some(data) = self.user_list.get_mut(cluster_name) {
            data.remove(user_name);
        }
    }

    // Expire LastWill
    pub fn add_expire_last_will(&self, expire_last_will: ExpireLastWill) {
        if let Some(data) = self
            .expire_last_wills
            .get_mut(&expire_last_will.cluster_name)
        {
            data.insert(expire_last_will.client_id.clone(), expire_last_will);
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(expire_last_will.client_id.clone(), expire_last_will.clone());
            self.expire_last_wills
                .insert(expire_last_will.cluster_name.clone(), data);
        }
    }

    pub fn remove_expire_last_will(&self, cluster_name: &str, client_id: &str) {
        if let Some(data) = self.expire_last_wills.get_mut(cluster_name) {
            data.remove(client_id);
        }
    }

    pub fn get_expire_last_wills(&self, cluster_name: &str) -> Vec<ExpireLastWill> {
        let mut results = Vec::new();
        if let Some(list) = self.expire_last_wills.get(cluster_name) {
            for raw in list.iter() {
                if is_send_last_will(raw.value()) {
                    results.push(raw.value().clone());
                }
            }
        }
        results
    }

    // Connector
    pub fn add_connector(&self, cluster_name: &str, connector: &MQTTConnector) {
        if let Some(data) = self.connector_list.get_mut(cluster_name) {
            data.insert(connector.connector_name.clone(), connector.clone());
        } else {
            let data = DashMap::with_capacity(8);
            data.insert(connector.connector_name.clone(), connector.clone());
            self.connector_list.insert(cluster_name.to_owned(), data);
        }
    }

    pub fn remove_connector(&self, cluster_name: &str, connector_name: &str) {
        if let Some(data) = self.connector_list.get_mut(cluster_name) {
            data.remove(connector_name);
        }
    }
    pub fn get_connector(&self, cluster_name: &str, connector_name: &str) -> Option<MQTTConnector> {
        if let Some(data) = self.connector_list.get(cluster_name) {
            if let Some(val) = data.get(connector_name) {
                return Some(val.clone());
            }
        }
        None
    }

    pub fn get_all_connector(&self) -> Vec<MQTTConnector> {
        let mut results = Vec::new();
        for (_, raw) in self.connector_list.clone() {
            for (_, val) in raw {
                results.push(val);
            }
        }
        results
    }

    // Report HeartBeart
    pub fn report_connector_heartbeat(
        &self,
        cluster_name: &str,
        connector_name: &str,
        heartbeat_time: u64,
    ) {
        let key = format!("{cluster_name}_{connector_name}");
        let heartbeat = ConnectorHeartbeat {
            cluster_name: cluster_name.to_owned(),
            connector_name: connector_name.to_owned(),
            last_heartbeat: heartbeat_time,
        };
        self.connector_heartbeat.insert(key, heartbeat);
    }

    pub fn remove_connector_heartbeat(&self, cluster_name: &str, connector_name: &str) {
        let key = format!("{cluster_name}_{connector_name}");
        self.connector_heartbeat.remove(&key);
    }

    pub fn get_all_connector_heartbeat(&self) -> Vec<ConnectorHeartbeat> {
        let mut results = Vec::new();
        for val in self.connector_heartbeat.clone() {
            results.push(val.1);
        }
        results
    }
}
