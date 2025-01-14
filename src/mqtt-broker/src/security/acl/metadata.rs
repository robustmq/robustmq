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

use crate::handler::connection_jitter::ConnectionJitterCondition;
use crate::handler::error::MqttBrokerError;
use common_base::enum_type::time_unit_enum::TimeUnit;
use common_base::tools::{convert_seconds, now_second};
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::{MqttAcl, MqttAclResourceType};
use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
use metadata_struct::mqtt::cluster::MqttClusterDynamicConnectionJitter;

#[derive(Clone)]
pub struct AclMetadata {
    // blacklist
    pub blacklist_user: DashMap<String, MqttAclBlackList>,
    pub blacklist_client_id: DashMap<String, MqttAclBlackList>,
    pub blacklist_ip: DashMap<String, MqttAclBlackList>,
    pub blacklist_user_match: DashMap<String, Vec<MqttAclBlackList>>,
    pub blacklist_client_id_match: DashMap<String, Vec<MqttAclBlackList>>,
    pub blacklist_ip_match: DashMap<String, Vec<MqttAclBlackList>>,

    // acl
    pub acl_user: DashMap<String, Vec<MqttAcl>>,
    pub acl_client_id: DashMap<String, Vec<MqttAcl>>,

    // connection jitter (client_id, ConnectionJitterCondition)
    pub connection_jitter_map: DashMap<String, ConnectionJitterCondition>,
}

impl Default for AclMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl AclMetadata {
    pub fn new() -> Self {
        AclMetadata {
            blacklist_user: DashMap::with_capacity(2),
            blacklist_client_id: DashMap::with_capacity(2),
            blacklist_ip: DashMap::with_capacity(2),
            blacklist_user_match: DashMap::with_capacity(2),
            blacklist_client_id_match: DashMap::with_capacity(2),
            blacklist_ip_match: DashMap::with_capacity(2),

            acl_user: DashMap::with_capacity(2),
            acl_client_id: DashMap::with_capacity(2),
            connection_jitter_map: DashMap::new(),
        }
    }

    pub fn get_connection_jitter_condition(
        &self,
        client_id: String,
    ) -> Option<ConnectionJitterCondition> {
        if let Some(connection_jitter_condition) = self.connection_jitter_map.get(&client_id) {
            return Some(connection_jitter_condition.clone());
        }
        None
    }

    pub fn add_connection_jitter_condition(
        &self,
        connection_jitter_condition: ConnectionJitterCondition,
    ) {
        self.connection_jitter_map.insert(
            connection_jitter_condition.client_id.clone(),
            connection_jitter_condition,
        );
    }

    pub fn remove_connection_jitter_condition(&self, client_id: &str) {
        self.connection_jitter_map.remove(client_id);
    }

    pub async fn remove_connection_jitter_conditions(
        &self,
        config: MqttClusterDynamicConnectionJitter,
    ) -> Result<(), MqttBrokerError> {
        let current_time = now_second();
        let window_time = convert_seconds(config.window_time, TimeUnit::Minutes) as u64;
        self.connection_jitter_map
            .retain(|_, connection_jitter_condition| {
                // we need retain elements within window_time,
                // so now_seconds - first_request_time must less than window_time
                current_time - connection_jitter_condition.first_request_time < window_time
            });
        Ok(())
    }

    pub fn parse_mqtt_acl(&self, acl: MqttAcl) {
        match acl.resource_type {
            MqttAclResourceType::ClientId => {
                if let Some(mut raw) = self.acl_client_id.get_mut(&acl.resource_name) {
                    raw.push(acl);
                } else {
                    self.acl_client_id
                        .insert(acl.resource_name.clone(), vec![acl]);
                }
            }
            MqttAclResourceType::User => {
                if let Some(mut raw) = self.acl_user.get_mut(&acl.resource_name) {
                    raw.push(acl);
                } else {
                    self.acl_user.insert(acl.resource_name.clone(), vec![acl]);
                }
            }
        }
    }

    pub fn remove_mqtt_acl(&self, acl: MqttAcl) {
        let resource_name = acl.resource_name.clone();
        match acl.resource_type {
            MqttAclResourceType::ClientId => {
                self.acl_client_id.remove(&resource_name);
            }
            MqttAclResourceType::User => {
                self.acl_user.remove(&resource_name);
            }
        }
    }

    pub fn parse_mqtt_blacklist(&self, blacklist: MqttAclBlackList) {
        match blacklist.blacklist_type {
            MqttAclBlackListType::ClientId => {
                self.blacklist_client_id
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            MqttAclBlackListType::User => {
                self.blacklist_user
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            MqttAclBlackListType::Ip => {
                self.blacklist_ip
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            MqttAclBlackListType::ClientIdMatch => {
                let key = self.get_client_id_match_key();
                if let Some(mut data) = self.blacklist_client_id_match.get_mut(&key) {
                    data.push(blacklist)
                } else {
                    self.blacklist_client_id_match.insert(key, vec![blacklist]);
                }
            }
            MqttAclBlackListType::UserMatch => {
                let key = self.get_user_match_key();
                if let Some(mut data) = self.blacklist_user_match.get_mut(&key) {
                    data.push(blacklist)
                } else {
                    self.blacklist_user_match.insert(key, vec![blacklist]);
                }
            }
            MqttAclBlackListType::IPCIDR => {
                let key = self.get_ip_cidr_key();
                if let Some(mut data) = self.blacklist_ip_match.get_mut(&key) {
                    data.push(blacklist)
                } else {
                    self.blacklist_ip_match.insert(key, vec![blacklist]);
                }
            }
        }
    }

    pub fn remove_mqtt_blacklist(&self, blacklist: MqttAclBlackList) {
        match blacklist.blacklist_type {
            MqttAclBlackListType::ClientId => {
                self.blacklist_client_id.remove(&blacklist.resource_name);
            }
            MqttAclBlackListType::User => {
                self.blacklist_user.remove(&blacklist.resource_name);
            }
            MqttAclBlackListType::Ip => {
                self.blacklist_ip.remove(&blacklist.resource_name);
            }
            MqttAclBlackListType::ClientIdMatch => {
                let key = self.get_client_id_match_key();
                self.blacklist_client_id_match.remove(&key);
            }
            MqttAclBlackListType::UserMatch => {
                let key = self.get_user_match_key();
                self.blacklist_user_match.remove(&key);
            }
            MqttAclBlackListType::IPCIDR => {
                let key = self.get_ip_cidr_key();
                self.blacklist_ip_match.remove(&key);
            }
        }
    }

    pub fn get_blacklist_user_match(&self) -> Option<Vec<MqttAclBlackList>> {
        let key = self.get_user_match_key();
        if let Some(data) = self.blacklist_user_match.get(&key) {
            return Some(data.clone());
        }
        None
    }

    pub fn get_blacklist_client_id_match(&self) -> Option<Vec<MqttAclBlackList>> {
        let key = self.get_client_id_match_key();
        if let Some(data) = self.blacklist_client_id_match.get(&key) {
            return Some(data.clone());
        }
        None
    }

    pub fn get_blacklist_ip_match(&self) -> Option<Vec<MqttAclBlackList>> {
        let key = self.get_ip_cidr_key();
        if let Some(data) = self.blacklist_ip_match.get(&key) {
            return Some(data.clone());
        }
        None
    }

    fn get_client_id_match_key(&self) -> String {
        "ClientIdMatch".to_string()
    }

    fn get_user_match_key(&self) -> String {
        "UserMatch".to_string()
    }

    fn get_ip_cidr_key(&self) -> String {
        "IPCIDR".to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::handler::connection_jitter::ConnectionJitterCondition;
    use crate::security::acl::metadata::AclMetadata;
    use common_base::tools::now_second;
    use metadata_struct::acl::mqtt_acl::{
        MqttAcl, MqttAclAction, MqttAclPermission, MqttAclResourceType,
    };
    use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
    use metadata_struct::mqtt::cluster::MqttClusterDynamicConnectionJitter;

    #[tokio::test]
    pub async fn test_mqtt_remove_connection_jitter() {
        let acl_metadata = AclMetadata::new();
        let condition1 = ConnectionJitterCondition {
            client_id: "test_id_1".to_string(),
            connect_times: 15,
            first_request_time: now_second() - 10,
        };
        let condition2 = ConnectionJitterCondition {
            client_id: "test_id_2".to_string(),
            connect_times: 15,
            first_request_time: now_second() - 70,
        };

        acl_metadata.add_connection_jitter_condition(condition1);

        acl_metadata.add_connection_jitter_condition(condition2);

        assert!(acl_metadata.connection_jitter_map.contains_key("test_id_1"));
        assert!(acl_metadata.connection_jitter_map.contains_key("test_id_2"));

        let jitter_config = MqttClusterDynamicConnectionJitter {
            enable: true,
            window_time: 1,
            max_client_connections: 15,
            ban_time: 5,
        };

        acl_metadata
            .remove_connection_jitter_conditions(jitter_config)
            .await
            .expect("TODO: panic message");

        assert!(acl_metadata.connection_jitter_map.contains_key("test_id_1"));
        assert!(!acl_metadata.connection_jitter_map.contains_key("test_id_2"));
    }

    #[tokio::test]
    pub async fn parse_mqtt_acl_test() {
        let acl_metadata = AclMetadata::new();
        // Test ClientId ACL
        let client_id_acl = MqttAcl {
            resource_type: MqttAclResourceType::ClientId,
            resource_name: "test_client".to_string(),
            topic: "".to_string(),
            ip: "".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Allow,
        };
        acl_metadata.parse_mqtt_acl(client_id_acl.clone());

        assert!(acl_metadata.acl_client_id.contains_key("test_client"));
        assert_eq!(
            acl_metadata.acl_client_id.get("test_client").unwrap().len(),
            1
        );
        assert_eq!(
            acl_metadata.acl_client_id.get("test_client").unwrap()[0].resource_name,
            "test_client"
        );

        // Test User ACL
        let user_acl = MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: "test_user".to_string(),
            topic: "".to_string(),
            ip: "".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Allow,
        };
        acl_metadata.parse_mqtt_acl(user_acl.clone());

        assert!(acl_metadata.acl_user.contains_key("test_user"));
        assert_eq!(acl_metadata.acl_user.get("test_user").unwrap().len(), 1);
        assert_eq!(
            acl_metadata.acl_user.get("test_user").unwrap()[0].resource_name,
            "test_user"
        );

        // Test multiple ACLs for the same ClientId
        acl_metadata.parse_mqtt_acl(client_id_acl);
        assert_eq!(
            acl_metadata.acl_client_id.get("test_client").unwrap().len(),
            2
        );

        // Test multiple ACLs for the same User
        acl_metadata.parse_mqtt_acl(user_acl);
        assert_eq!(acl_metadata.acl_user.get("test_user").unwrap().len(), 2);
    }
    #[tokio::test]
    pub async fn parse_mqtt_blacklist_test() {
        let acl_metadata = AclMetadata::new();

        // Test ClientId blacklist
        let client_id_blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::ClientId,
            resource_name: "test_client".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(client_id_blacklist);
        assert!(acl_metadata.blacklist_client_id.contains_key("test_client"));

        // Test User blacklist
        let user_blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::User,
            resource_name: "test_user".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(user_blacklist);
        assert!(acl_metadata.blacklist_user.contains_key("test_user"));

        // Test IP blacklist
        let ip_blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::Ip,
            resource_name: "192.168.1.1".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(ip_blacklist);
        assert!(acl_metadata.blacklist_ip.contains_key("192.168.1.1"));

        // Test ClientIdMatch blacklist
        let client_id_match_blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::ClientIdMatch,
            resource_name: "test_client_*".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(client_id_match_blacklist);
        let client_id_match_key = acl_metadata.get_client_id_match_key();
        assert!(acl_metadata
            .blacklist_client_id_match
            .contains_key(&client_id_match_key));
        assert_eq!(
            acl_metadata
                .blacklist_client_id_match
                .get(&client_id_match_key)
                .unwrap()
                .len(),
            1
        );

        // Test UserMatch blacklist
        let user_match_blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::UserMatch,
            resource_name: "test_user_*".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(user_match_blacklist);
        let user_match_key = acl_metadata.get_user_match_key();
        assert!(acl_metadata
            .blacklist_user_match
            .contains_key(&user_match_key));
        assert_eq!(
            acl_metadata
                .blacklist_user_match
                .get(&user_match_key)
                .unwrap()
                .len(),
            1
        );

        // Test IPCIDR blacklist
        let ip_cidr_blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::IPCIDR,
            resource_name: "192.168.1.0/24".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(ip_cidr_blacklist);
        let ip_cidr_key = acl_metadata.get_ip_cidr_key();
        assert!(acl_metadata.blacklist_ip_match.contains_key(&ip_cidr_key));
        assert_eq!(
            acl_metadata
                .blacklist_ip_match
                .get(&ip_cidr_key)
                .unwrap()
                .len(),
            1
        );

        // Test adding multiple entries for match types
        let another_client_id_match_blacklist = MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::ClientIdMatch,
            resource_name: "another_client_*".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(another_client_id_match_blacklist);
        assert_eq!(
            acl_metadata
                .blacklist_client_id_match
                .get(&client_id_match_key)
                .unwrap()
                .len(),
            2
        );
    }
}
