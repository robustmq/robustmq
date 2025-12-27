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

use crate::controller::session_expire::ExpireLastWill;
use crate::core::cache::CacheManager;
use crate::server::services::mqtt::connector::ConnectorHeartbeat;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::user::MqttUser;

impl CacheManager {
    pub fn add_topic(&self, topic: MQTTTopic) {
        self.topic_list.insert(topic.topic_name.clone(), topic);
    }

    pub fn remove_topic(&self, topic_name: &str) {
        self.topic_list.remove(topic_name);
    }

    pub fn add_user(&self, user: MqttUser) {
        self.user_list.insert(user.username.clone(), user);
    }

    pub fn remove_user(&self, user_name: &str) {
        self.user_list.remove(user_name);
    }

    pub fn add_expire_last_will(&self, expire_last_will: ExpireLastWill) {
        self.expire_last_wills
            .insert(expire_last_will.client_id.clone(), expire_last_will);
    }

    pub fn remove_expire_last_will(&self, client_id: &str) {
        self.expire_last_wills.remove(client_id);
    }

    pub fn get_expire_last_wills(&self) -> Vec<ExpireLastWill> {
        self.expire_last_wills
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
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
