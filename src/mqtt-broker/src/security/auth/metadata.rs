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

use crate::core::flapping_detect::FlappingDetectCondition;
use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use common_base::enum_type::time_unit_enum::TimeUnit;
use common_base::tools::{convert_seconds, now_second};
use common_config::config::MqttFlappingDetect;
use dashmap::DashMap;
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;

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

    // connection jitter (client_id, FlappingDetectCondition)
    pub flapping_detect_map: DashMap<String, FlappingDetectCondition>,
}

impl Default for AclMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl AclMetadata {
    const CLIENT_ID_MATCH_KEY: &'static str = "ClientIdMatch";
    const USER_MATCH_KEY: &'static str = "UserMatch";
    const IP_CIDR_KEY: &'static str = "IPCIDR";

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
            flapping_detect_map: DashMap::new(),
        }
    }

    // Flapping Detects
    pub fn get_flapping_detect_condition(
        &self,
        client_id: &str,
    ) -> Option<FlappingDetectCondition> {
        if let Some(flapping_detect_condition) = self.flapping_detect_map.get(client_id) {
            return Some(flapping_detect_condition.clone());
        }
        None
    }

    pub fn add_flapping_detect_condition(
        &self,
        flapping_detect_condition: FlappingDetectCondition,
    ) {
        self.flapping_detect_map.insert(
            flapping_detect_condition.client_id.clone(),
            flapping_detect_condition,
        );
    }

    pub fn remove_flapping_detect_condition(&self, client_id: &str) {
        self.flapping_detect_map.remove(client_id);
    }

    pub async fn remove_flapping_detect_conditions(&self, config: MqttFlappingDetect) {
        let current_time = now_second();
        let window_time = convert_seconds(config.window_time as u64, TimeUnit::Minutes);
        self.flapping_detect_map
            .retain(|_, flapping_detect_condition| {
                // we need retain elements within window_time,
                // so now_seconds - first_request_time must less than window_time
                current_time - flapping_detect_condition.first_request_time < window_time
            });
    }

    // ACL
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
        match acl.resource_type {
            MqttAclResourceType::ClientId => {
                let mut remove_key = false;
                if let Some(mut list) = self.acl_client_id.get_mut(&acl.resource_name) {
                    if let Some(pos) = list.iter().position(|item| item == &acl) {
                        list.remove(pos);
                    }
                    remove_key = list.is_empty();
                }
                if remove_key {
                    self.acl_client_id.remove(&acl.resource_name);
                }
            }
            MqttAclResourceType::User => {
                let mut remove_key = false;
                if let Some(mut list) = self.acl_user.get_mut(&acl.resource_name) {
                    if let Some(pos) = list.iter().position(|item| item == &acl) {
                        list.remove(pos);
                    }
                    remove_key = list.is_empty();
                }
                if remove_key {
                    self.acl_user.remove(&acl.resource_name);
                }
            }
        }
    }

    pub fn get_all_acl(&self) -> Vec<MqttAcl> {
        let mut data: Vec<MqttAcl> = Vec::new();
        for entry in self.acl_user.iter() {
            data.extend(entry.value().iter().cloned());
        }
        for entry in self.acl_client_id.iter() {
            data.extend(entry.value().iter().cloned());
        }
        data.sort_by(|a, b| {
            (
                a.resource_type.to_string(),
                a.resource_name.clone(),
                a.topic.clone(),
                a.ip.clone(),
                a.action.to_string(),
                a.permission.to_string(),
            )
                .cmp(&(
                    b.resource_type.to_string(),
                    b.resource_name.clone(),
                    b.topic.clone(),
                    b.ip.clone(),
                    b.action.to_string(),
                    b.permission.to_string(),
                ))
        });
        data
    }

    // Blacklist
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
                if let Some(mut data) = self.blacklist_client_id_match.get_mut(key) {
                    data.push(blacklist)
                } else {
                    self.blacklist_client_id_match
                        .insert(key.to_string(), vec![blacklist]);
                }
            }
            MqttAclBlackListType::UserMatch => {
                let key = self.get_user_match_key();
                if let Some(mut data) = self.blacklist_user_match.get_mut(key) {
                    data.push(blacklist)
                } else {
                    self.blacklist_user_match
                        .insert(key.to_string(), vec![blacklist]);
                }
            }
            MqttAclBlackListType::IPCIDR => {
                let key = self.get_ip_cidr_key();
                if let Some(mut data) = self.blacklist_ip_match.get_mut(key) {
                    data.push(blacklist)
                } else {
                    self.blacklist_ip_match
                        .insert(key.to_string(), vec![blacklist]);
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
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_client_id_match.get_mut(key) {
                    if let Some(pos) = data
                        .iter()
                        .position(|item| item.resource_name == blacklist.resource_name)
                    {
                        data.remove(pos);
                    }
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_client_id_match.remove(key);
                }
            }
            MqttAclBlackListType::UserMatch => {
                let key = self.get_user_match_key();
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_user_match.get_mut(key) {
                    if let Some(pos) = data
                        .iter()
                        .position(|item| item.resource_name == blacklist.resource_name)
                    {
                        data.remove(pos);
                    }
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_user_match.remove(key);
                }
            }
            MqttAclBlackListType::IPCIDR => {
                let key = self.get_ip_cidr_key();
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_ip_match.get_mut(key) {
                    if let Some(pos) = data
                        .iter()
                        .position(|item| item.resource_name == blacklist.resource_name)
                    {
                        data.remove(pos);
                    }
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_ip_match.remove(key);
                }
            }
        }
    }

    pub fn get_blacklist_user_match(&self) -> Option<Vec<MqttAclBlackList>> {
        let key = self.get_user_match_key();
        if let Some(data) = self.blacklist_user_match.get(key) {
            return Some(data.clone());
        }
        None
    }

    pub fn get_blacklist_client_id_match(&self) -> Option<Vec<MqttAclBlackList>> {
        let key = self.get_client_id_match_key();
        if let Some(data) = self.blacklist_client_id_match.get(key) {
            return Some(data.clone());
        }
        None
    }

    pub fn get_blacklist_ip_match(&self) -> Option<Vec<MqttAclBlackList>> {
        let key = self.get_ip_cidr_key();
        if let Some(data) = self.blacklist_ip_match.get(key) {
            return Some(data.clone());
        }
        None
    }

    pub fn get_all_blacklist(&self) -> Vec<MqttAclBlackList> {
        let mut data: Vec<MqttAclBlackList> = Vec::new();
        data.extend(
            self.blacklist_user
                .iter()
                .map(|entry| entry.value().clone()),
        );
        data.extend(
            self.blacklist_client_id
                .iter()
                .map(|entry| entry.value().clone()),
        );
        data.extend(self.blacklist_ip.iter().map(|entry| entry.value().clone()));
        for entry in self.blacklist_user_match.iter() {
            data.extend(entry.value().iter().cloned());
        }
        for entry in self.blacklist_client_id_match.iter() {
            data.extend(entry.value().iter().cloned());
        }
        for entry in self.blacklist_ip_match.iter() {
            data.extend(entry.value().iter().cloned());
        }
        data.sort_by(|a, b| {
            (
                a.blacklist_type.to_string(),
                a.resource_name.clone(),
                a.end_time,
                a.desc.clone(),
            )
                .cmp(&(
                    b.blacklist_type.to_string(),
                    b.resource_name.clone(),
                    b.end_time,
                    b.desc.clone(),
                ))
        });
        data
    }

    fn get_client_id_match_key(&self) -> &'static str {
        Self::CLIENT_ID_MATCH_KEY
    }

    fn get_user_match_key(&self) -> &'static str {
        Self::USER_MATCH_KEY
    }

    fn get_ip_cidr_key(&self) -> &'static str {
        Self::IP_CIDR_KEY
    }
}

#[cfg(test)]
mod test {
    use crate::core::flapping_detect::FlappingDetectCondition;
    use crate::security::auth::metadata::AclMetadata;
    use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
    use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
    use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
    use common_base::tools::now_second;
    use common_config::config::MqttFlappingDetect;
    use metadata_struct::acl::mqtt_acl::MqttAcl;
    use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;

    #[tokio::test]
    pub async fn test_mqtt_remove_flapping_detect() {
        let acl_metadata = AclMetadata::new();
        let condition1 = FlappingDetectCondition {
            client_id: "test_id_1".to_string(),
            before_last_window_connections: 15,
            first_request_time: now_second() - 10,
        };
        let condition2 = FlappingDetectCondition {
            client_id: "test_id_2".to_string(),
            before_last_window_connections: 15,
            first_request_time: now_second() - 70,
        };

        acl_metadata.add_flapping_detect_condition(condition1);

        acl_metadata.add_flapping_detect_condition(condition2);

        assert!(acl_metadata.flapping_detect_map.contains_key("test_id_1"));
        assert!(acl_metadata.flapping_detect_map.contains_key("test_id_2"));

        let jitter_config = MqttFlappingDetect {
            enable: true,
            window_time: 1,
            max_client_connections: 15,
            ban_time: 5,
        };

        acl_metadata
            .remove_flapping_detect_conditions(jitter_config)
            .await;

        assert!(acl_metadata.flapping_detect_map.contains_key("test_id_1"));
        assert!(!acl_metadata.flapping_detect_map.contains_key("test_id_2"));
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

        // Remove only one acl item, keep the other one
        acl_metadata.remove_mqtt_acl(MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: "test_user".to_string(),
            topic: "".to_string(),
            ip: "".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Allow,
        });
        assert_eq!(acl_metadata.acl_user.get("test_user").unwrap().len(), 1);
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
            .contains_key(client_id_match_key));
        assert_eq!(
            acl_metadata
                .blacklist_client_id_match
                .get(client_id_match_key)
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
            .contains_key(user_match_key));
        assert_eq!(
            acl_metadata
                .blacklist_user_match
                .get(user_match_key)
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
        assert!(acl_metadata.blacklist_ip_match.contains_key(ip_cidr_key));
        assert_eq!(
            acl_metadata
                .blacklist_ip_match
                .get(ip_cidr_key)
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
                .get(client_id_match_key)
                .unwrap()
                .len(),
            2
        );

        let all_blacklist = acl_metadata.get_all_blacklist();
        assert_eq!(all_blacklist.len(), 7);

        // Remove one match rule should not remove entire match group
        acl_metadata.remove_mqtt_blacklist(MqttAclBlackList {
            blacklist_type: MqttAclBlackListType::ClientIdMatch,
            resource_name: "test_client_*".to_string(),
            end_time: 0,
            desc: "".to_string(),
        });
        assert_eq!(
            acl_metadata
                .blacklist_client_id_match
                .get(client_id_match_key)
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    pub async fn get_all_acl_test() {
        let acl_metadata = AclMetadata::new();
        acl_metadata.parse_mqtt_acl(MqttAcl {
            resource_type: MqttAclResourceType::ClientId,
            resource_name: "client-a".to_string(),
            topic: "a/#".to_string(),
            ip: "".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Allow,
        });
        acl_metadata.parse_mqtt_acl(MqttAcl {
            resource_type: MqttAclResourceType::User,
            resource_name: "user-a".to_string(),
            topic: "b/#".to_string(),
            ip: "".to_string(),
            action: MqttAclAction::Publish,
            permission: MqttAclPermission::Deny,
        });

        let all_acl = acl_metadata.get_all_acl();
        assert_eq!(all_acl.len(), 2);
    }
}
