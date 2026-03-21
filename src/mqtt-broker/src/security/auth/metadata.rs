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
    // blacklist exact match: outer = tenant, inner = resource_name
    pub blacklist_user: DashMap<String, DashMap<String, MqttAclBlackList>>,
    pub blacklist_client_id: DashMap<String, DashMap<String, MqttAclBlackList>>,
    pub blacklist_ip: DashMap<String, DashMap<String, MqttAclBlackList>>,
    // blacklist wildcard match: outer = tenant, value = list of patterns
    pub blacklist_user_match: DashMap<String, Vec<MqttAclBlackList>>,
    pub blacklist_client_id_match: DashMap<String, Vec<MqttAclBlackList>>,
    pub blacklist_ip_match: DashMap<String, Vec<MqttAclBlackList>>,

    // acl: outer = tenant, inner = resource_name
    pub acl_user: DashMap<String, DashMap<String, Vec<MqttAcl>>>,
    pub acl_client_id: DashMap<String, DashMap<String, Vec<MqttAcl>>>,

    // connection jitter: outer = tenant, inner = (client_id, FlappingDetectCondition)
    pub flapping_detect_map: DashMap<String, DashMap<String, FlappingDetectCondition>>,
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
            flapping_detect_map: DashMap::new(),
        }
    }

    // Flapping Detects
    pub fn get_flapping_detect_condition(
        &self,
        tenant: &str,
        client_id: &str,
    ) -> Option<FlappingDetectCondition> {
        self.flapping_detect_map
            .get(tenant)
            .and_then(|inner| inner.get(client_id).map(|v| v.clone()))
    }

    pub fn add_flapping_detect_condition(
        &self,
        flapping_detect_condition: FlappingDetectCondition,
    ) {
        self.flapping_detect_map
            .entry(flapping_detect_condition.tenant.clone())
            .or_default()
            .insert(
                flapping_detect_condition.client_id.clone(),
                flapping_detect_condition,
            );
    }

    pub fn remove_flapping_detect_condition(&self, tenant: &str, client_id: &str) {
        if let Some(inner) = self.flapping_detect_map.get(tenant) {
            inner.remove(client_id);
        }
    }

    pub async fn remove_flapping_detect_conditions(&self, config: MqttFlappingDetect) {
        let current_time = now_second();
        let window_time = convert_seconds(config.window_time as u64, TimeUnit::Minutes);
        for tenant_entry in self.flapping_detect_map.iter() {
            tenant_entry.value().retain(|_, flapping_detect_condition| {
                // we need retain elements within window_time,
                // so now_seconds - first_request_time must less than window_time
                current_time - flapping_detect_condition.first_request_time < window_time
            });
        }
    }

    // ACL
    pub fn parse_mqtt_acl(&self, acl: MqttAcl) {
        let map = match acl.resource_type {
            MqttAclResourceType::ClientId => &self.acl_client_id,
            MqttAclResourceType::User => &self.acl_user,
        };
        map.entry(acl.tenant.clone()).or_insert_with(DashMap::new);
        if let Some(tenant_map) = map.get(&acl.tenant) {
            if let Some(mut list) = tenant_map.get_mut(&acl.resource_name) {
                list.push(acl);
            } else {
                tenant_map.insert(acl.resource_name.clone(), vec![acl]);
            }
        }
    }

    pub fn remove_mqtt_acl(&self, acl: MqttAcl) {
        for map in [&self.acl_client_id, &self.acl_user] {
            if let Some(tenant_map) = map.get(&acl.tenant) {
                let mut keys_to_clean = Vec::new();
                for mut entry in tenant_map.iter_mut() {
                    entry.value_mut().retain(|item| item.name != acl.name);
                    if entry.value().is_empty() {
                        keys_to_clean.push(entry.key().clone());
                    }
                }
                for key in keys_to_clean {
                    tenant_map.remove(&key);
                }
            }
        }
    }

    pub fn get_all_acl(&self) -> Vec<MqttAcl> {
        let mut data: Vec<MqttAcl> = Vec::new();
        for tenant_entry in self.acl_user.iter() {
            for acl_entry in tenant_entry.value().iter() {
                data.extend(acl_entry.value().iter().cloned());
            }
        }
        for tenant_entry in self.acl_client_id.iter() {
            for acl_entry in tenant_entry.value().iter() {
                data.extend(acl_entry.value().iter().cloned());
            }
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

    pub fn get_acl_by_tenant(&self, tenant: &str) -> Vec<MqttAcl> {
        let mut data: Vec<MqttAcl> = Vec::new();
        if let Some(user_map) = self.acl_user.get(tenant) {
            for acl_entry in user_map.iter() {
                data.extend(acl_entry.value().iter().cloned());
            }
        }
        if let Some(client_map) = self.acl_client_id.get(tenant) {
            for acl_entry in client_map.iter() {
                data.extend(acl_entry.value().iter().cloned());
            }
        }
        data
    }

    // Blacklist
    pub fn parse_mqtt_blacklist(&self, blacklist: MqttAclBlackList) {
        match blacklist.blacklist_type {
            MqttAclBlackListType::ClientId => {
                self.blacklist_client_id
                    .entry(blacklist.tenant.clone())
                    .or_insert_with(DashMap::new)
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            MqttAclBlackListType::User => {
                self.blacklist_user
                    .entry(blacklist.tenant.clone())
                    .or_insert_with(DashMap::new)
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            MqttAclBlackListType::Ip => {
                self.blacklist_ip
                    .entry(blacklist.tenant.clone())
                    .or_insert_with(DashMap::new)
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            MqttAclBlackListType::ClientIdMatch => {
                if let Some(mut data) = self.blacklist_client_id_match.get_mut(&blacklist.tenant) {
                    data.push(blacklist);
                } else {
                    let tenant = blacklist.tenant.clone();
                    self.blacklist_client_id_match
                        .insert(tenant, vec![blacklist]);
                }
            }
            MqttAclBlackListType::UserMatch => {
                if let Some(mut data) = self.blacklist_user_match.get_mut(&blacklist.tenant) {
                    data.push(blacklist);
                } else {
                    let tenant = blacklist.tenant.clone();
                    self.blacklist_user_match.insert(tenant, vec![blacklist]);
                }
            }
            MqttAclBlackListType::IPCIDR => {
                if let Some(mut data) = self.blacklist_ip_match.get_mut(&blacklist.tenant) {
                    data.push(blacklist);
                } else {
                    let tenant = blacklist.tenant.clone();
                    self.blacklist_ip_match.insert(tenant, vec![blacklist]);
                }
            }
        }
    }

    pub fn remove_mqtt_blacklist(&self, blacklist: MqttAclBlackList) {
        match blacklist.blacklist_type {
            MqttAclBlackListType::ClientId => {
                if let Some(tenant_map) = self.blacklist_client_id.get(&blacklist.tenant) {
                    tenant_map.remove(&blacklist.resource_name);
                }
            }
            MqttAclBlackListType::User => {
                if let Some(tenant_map) = self.blacklist_user.get(&blacklist.tenant) {
                    tenant_map.remove(&blacklist.resource_name);
                }
            }
            MqttAclBlackListType::Ip => {
                if let Some(tenant_map) = self.blacklist_ip.get(&blacklist.tenant) {
                    tenant_map.remove(&blacklist.resource_name);
                }
            }
            MqttAclBlackListType::ClientIdMatch => {
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_client_id_match.get_mut(&blacklist.tenant) {
                    if let Some(pos) = data
                        .iter()
                        .position(|item| item.resource_name == blacklist.resource_name)
                    {
                        data.remove(pos);
                    }
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_client_id_match.remove(&blacklist.tenant);
                }
            }
            MqttAclBlackListType::UserMatch => {
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_user_match.get_mut(&blacklist.tenant) {
                    if let Some(pos) = data
                        .iter()
                        .position(|item| item.resource_name == blacklist.resource_name)
                    {
                        data.remove(pos);
                    }
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_user_match.remove(&blacklist.tenant);
                }
            }
            MqttAclBlackListType::IPCIDR => {
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_ip_match.get_mut(&blacklist.tenant) {
                    if let Some(pos) = data
                        .iter()
                        .position(|item| item.resource_name == blacklist.resource_name)
                    {
                        data.remove(pos);
                    }
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_ip_match.remove(&blacklist.tenant);
                }
            }
        }
    }

    /// Returns all wildcard user blacklist entries across all tenants.
    pub fn get_blacklist_user_match(&self) -> Vec<MqttAclBlackList> {
        self.blacklist_user_match
            .iter()
            .flat_map(|entry| entry.value().clone())
            .collect()
    }

    /// Returns all wildcard client-id blacklist entries across all tenants.
    pub fn get_blacklist_client_id_match(&self) -> Vec<MqttAclBlackList> {
        self.blacklist_client_id_match
            .iter()
            .flat_map(|entry| entry.value().clone())
            .collect()
    }

    /// Returns all CIDR/wildcard IP blacklist entries across all tenants.
    pub fn get_blacklist_ip_match(&self) -> Vec<MqttAclBlackList> {
        self.blacklist_ip_match
            .iter()
            .flat_map(|entry| entry.value().clone())
            .collect()
    }

    pub fn get_all_blacklist(&self) -> Vec<MqttAclBlackList> {
        let mut data: Vec<MqttAclBlackList> = Vec::new();
        for tenant_entry in self.blacklist_user.iter() {
            data.extend(tenant_entry.value().iter().map(|e| e.value().clone()));
        }
        for tenant_entry in self.blacklist_client_id.iter() {
            data.extend(tenant_entry.value().iter().map(|e| e.value().clone()));
        }
        for tenant_entry in self.blacklist_ip.iter() {
            data.extend(tenant_entry.value().iter().map(|e| e.value().clone()));
        }
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

    pub fn get_blacklist_by_tenant(&self, tenant: &str) -> Vec<MqttAclBlackList> {
        let mut data: Vec<MqttAclBlackList> = Vec::new();
        if let Some(m) = self.blacklist_user.get(tenant) {
            data.extend(m.iter().map(|e| e.value().clone()));
        }
        if let Some(m) = self.blacklist_client_id.get(tenant) {
            data.extend(m.iter().map(|e| e.value().clone()));
        }
        if let Some(m) = self.blacklist_ip.get(tenant) {
            data.extend(m.iter().map(|e| e.value().clone()));
        }
        if let Some(v) = self.blacklist_user_match.get(tenant) {
            data.extend(v.iter().cloned());
        }
        if let Some(v) = self.blacklist_client_id_match.get(tenant) {
            data.extend(v.iter().cloned());
        }
        if let Some(v) = self.blacklist_ip_match.get(tenant) {
            data.extend(v.iter().cloned());
        }
        data
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
    use metadata_struct::tenant::DEFAULT_TENANT;

    const TENANT: &str = DEFAULT_TENANT;

    #[tokio::test]
    pub async fn test_mqtt_remove_flapping_detect() {
        let acl_metadata = AclMetadata::new();
        let condition1 = FlappingDetectCondition {
            tenant: "default".to_string(),
            client_id: "test_id_1".to_string(),
            before_last_window_connections: 15,
            first_request_time: now_second() - 10,
        };
        let condition2 = FlappingDetectCondition {
            tenant: "default".to_string(),
            client_id: "test_id_2".to_string(),
            before_last_window_connections: 15,
            first_request_time: now_second() - 70,
        };

        acl_metadata.add_flapping_detect_condition(condition1);
        acl_metadata.add_flapping_detect_condition(condition2);

        assert!(acl_metadata
            .flapping_detect_map
            .get("default")
            .map(|m| m.contains_key("test_id_1"))
            .unwrap_or(false));
        assert!(acl_metadata
            .flapping_detect_map
            .get("default")
            .map(|m| m.contains_key("test_id_2"))
            .unwrap_or(false));

        let jitter_config = MqttFlappingDetect {
            enable: true,
            window_time: 1,
            max_client_connections: 15,
            ban_time: 5,
        };

        acl_metadata
            .remove_flapping_detect_conditions(jitter_config)
            .await;

        assert!(acl_metadata
            .flapping_detect_map
            .get("default")
            .map(|m| m.contains_key("test_id_1"))
            .unwrap_or(false));
        assert!(!acl_metadata
            .flapping_detect_map
            .get("default")
            .map(|m| m.contains_key("test_id_2"))
            .unwrap_or(false));
    }

    #[tokio::test]
    pub async fn parse_mqtt_acl_test() {
        let acl_metadata = AclMetadata::new();
        // Test ClientId ACL
        let client_id_acl = MqttAcl {
            name: "acl-client-1".to_string(),
            desc: String::new(),
            tenant: TENANT.to_string(),
            resource_type: MqttAclResourceType::ClientId,
            resource_name: "test_client".to_string(),
            topic: "".to_string(),
            ip: "".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Allow,
        };
        acl_metadata.parse_mqtt_acl(client_id_acl.clone());

        assert!(acl_metadata
            .acl_client_id
            .get(TENANT)
            .map(|m| m.contains_key("test_client"))
            .unwrap_or(false));
        assert_eq!(
            acl_metadata
                .acl_client_id
                .get(TENANT)
                .and_then(|m| m.get("test_client").map(|v| v.len()))
                .unwrap_or(0),
            1
        );

        // Test User ACL
        let user_acl = MqttAcl {
            name: "acl-user-1".to_string(),
            desc: String::new(),
            tenant: TENANT.to_string(),
            resource_type: MqttAclResourceType::User,
            resource_name: "test_user".to_string(),
            topic: "".to_string(),
            ip: "".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Allow,
        };
        acl_metadata.parse_mqtt_acl(user_acl.clone());

        assert!(acl_metadata
            .acl_user
            .get(TENANT)
            .map(|m| m.contains_key("test_user"))
            .unwrap_or(false));
        assert_eq!(
            acl_metadata
                .acl_user
                .get(TENANT)
                .and_then(|m| m.get("test_user").map(|v| v.len()))
                .unwrap_or(0),
            1
        );

        // Test multiple ACLs for same ClientId
        let client_id_acl2 = MqttAcl {
            name: "acl-client-2".to_string(),
            ..client_id_acl
        };
        acl_metadata.parse_mqtt_acl(client_id_acl2);
        assert_eq!(
            acl_metadata
                .acl_client_id
                .get(TENANT)
                .and_then(|m| m.get("test_client").map(|v| v.len()))
                .unwrap_or(0),
            2
        );

        // Test multiple ACLs for same User
        let user_acl2 = MqttAcl {
            name: "acl-user-2".to_string(),
            ..user_acl.clone()
        };
        acl_metadata.parse_mqtt_acl(user_acl2);
        assert_eq!(
            acl_metadata
                .acl_user
                .get(TENANT)
                .and_then(|m| m.get("test_user").map(|v| v.len()))
                .unwrap_or(0),
            2
        );

        // Remove only one acl item by name, keep the other one
        acl_metadata.remove_mqtt_acl(MqttAcl {
            name: "acl-user-1".to_string(),
            desc: String::new(),
            tenant: TENANT.to_string(),
            resource_type: MqttAclResourceType::User,
            resource_name: "test_user".to_string(),
            topic: "".to_string(),
            ip: "".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Allow,
        });
        assert_eq!(
            acl_metadata
                .acl_user
                .get(TENANT)
                .and_then(|m| m.get("test_user").map(|v| v.len()))
                .unwrap_or(0),
            1
        );
    }

    #[tokio::test]
    pub async fn parse_mqtt_blacklist_test() {
        let acl_metadata = AclMetadata::new();

        // Test ClientId blacklist
        let client_id_blacklist = MqttAclBlackList {
            tenant: TENANT.to_string(),
            blacklist_type: MqttAclBlackListType::ClientId,
            resource_name: "test_client".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(client_id_blacklist);
        assert!(acl_metadata
            .blacklist_client_id
            .get(TENANT)
            .map(|m| m.contains_key("test_client"))
            .unwrap_or(false));

        // Test User blacklist
        let user_blacklist = MqttAclBlackList {
            tenant: TENANT.to_string(),
            blacklist_type: MqttAclBlackListType::User,
            resource_name: "test_user".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(user_blacklist);
        assert!(acl_metadata
            .blacklist_user
            .get(TENANT)
            .map(|m| m.contains_key("test_user"))
            .unwrap_or(false));

        // Test IP blacklist
        let ip_blacklist = MqttAclBlackList {
            tenant: TENANT.to_string(),
            blacklist_type: MqttAclBlackListType::Ip,
            resource_name: "192.168.1.1".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(ip_blacklist);
        assert!(acl_metadata
            .blacklist_ip
            .get(TENANT)
            .map(|m| m.contains_key("192.168.1.1"))
            .unwrap_or(false));

        // Test ClientIdMatch blacklist
        let client_id_match_blacklist = MqttAclBlackList {
            tenant: TENANT.to_string(),
            blacklist_type: MqttAclBlackListType::ClientIdMatch,
            resource_name: "test_client_*".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(client_id_match_blacklist);
        assert!(acl_metadata.blacklist_client_id_match.contains_key(TENANT));
        assert_eq!(
            acl_metadata
                .blacklist_client_id_match
                .get(TENANT)
                .map(|v| v.len())
                .unwrap_or(0),
            1
        );

        // Test UserMatch blacklist
        let user_match_blacklist = MqttAclBlackList {
            tenant: TENANT.to_string(),
            blacklist_type: MqttAclBlackListType::UserMatch,
            resource_name: "test_user_*".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(user_match_blacklist);
        assert!(acl_metadata.blacklist_user_match.contains_key(TENANT));
        assert_eq!(
            acl_metadata
                .blacklist_user_match
                .get(TENANT)
                .map(|v| v.len())
                .unwrap_or(0),
            1
        );

        // Test IPCIDR blacklist
        let ip_cidr_blacklist = MqttAclBlackList {
            tenant: TENANT.to_string(),
            blacklist_type: MqttAclBlackListType::IPCIDR,
            resource_name: "192.168.1.0/24".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(ip_cidr_blacklist);
        assert!(acl_metadata.blacklist_ip_match.contains_key(TENANT));
        assert_eq!(
            acl_metadata
                .blacklist_ip_match
                .get(TENANT)
                .map(|v| v.len())
                .unwrap_or(0),
            1
        );

        // Test adding multiple entries for match types
        let another_client_id_match_blacklist = MqttAclBlackList {
            tenant: TENANT.to_string(),
            blacklist_type: MqttAclBlackListType::ClientIdMatch,
            resource_name: "another_client_*".to_string(),
            end_time: now_second() + 100,
            desc: "".to_string(),
        };
        acl_metadata.parse_mqtt_blacklist(another_client_id_match_blacklist);
        assert_eq!(
            acl_metadata
                .blacklist_client_id_match
                .get(TENANT)
                .map(|v| v.len())
                .unwrap_or(0),
            2
        );

        let all_blacklist = acl_metadata.get_all_blacklist();
        assert_eq!(all_blacklist.len(), 7);

        // Remove one match rule should not remove entire match group
        acl_metadata.remove_mqtt_blacklist(MqttAclBlackList {
            tenant: TENANT.to_string(),
            blacklist_type: MqttAclBlackListType::ClientIdMatch,
            resource_name: "test_client_*".to_string(),
            end_time: 0,
            desc: "".to_string(),
        });
        assert_eq!(
            acl_metadata
                .blacklist_client_id_match
                .get(TENANT)
                .map(|v| v.len())
                .unwrap_or(0),
            1
        );
    }

    #[tokio::test]
    pub async fn get_all_acl_test() {
        let acl_metadata = AclMetadata::new();
        acl_metadata.parse_mqtt_acl(MqttAcl {
            name: "acl-get-all-1".to_string(),
            desc: String::new(),
            tenant: TENANT.to_string(),
            resource_type: MqttAclResourceType::ClientId,
            resource_name: "client-a".to_string(),
            topic: "a/#".to_string(),
            ip: "".to_string(),
            action: MqttAclAction::All,
            permission: MqttAclPermission::Allow,
        });
        acl_metadata.parse_mqtt_acl(MqttAcl {
            name: "acl-get-all-2".to_string(),
            desc: String::new(),
            tenant: TENANT.to_string(),
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
