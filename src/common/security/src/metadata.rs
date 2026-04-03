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

use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
use dashmap::DashMap;
use metadata_struct::auth::acl::SecurityAcl;
use metadata_struct::auth::blacklist::SecurityBlackList;
use metadata_struct::auth::user::SecurityUser;
use metadata_struct::mqtt::auth::authn_config::AuthnConfig;

#[derive(Clone)]
pub struct SecurityMetadata {
    pub authn_list: DashMap<String, AuthnConfig>,

    // ==== User  ====
    // (tenant, (username, User))
    pub user_info: DashMap<String, DashMap<String, SecurityUser>>,

    // ==== Acl  ====
    // (tenant, (user, Vec<SecurityAcl>)
    pub acl_user: DashMap<String, DashMap<String, Vec<SecurityAcl>>>,

    // (tenant, (client_id, Vec<SecurityAcl>)
    pub acl_client_id: DashMap<String, DashMap<String, Vec<SecurityAcl>>>,

    // ==== BlackList ====
    // (tenant, （resource_name, SecurityBlackList）)
    pub blacklist_user: DashMap<String, DashMap<String, SecurityBlackList>>,
    pub blacklist_client_id: DashMap<String, DashMap<String, SecurityBlackList>>,
    pub blacklist_ip: DashMap<String, DashMap<String, SecurityBlackList>>,

    // (tenant, Vec<SecurityBlackList>)
    pub blacklist_user_match: DashMap<String, Vec<SecurityBlackList>>,
    pub blacklist_client_id_match: DashMap<String, Vec<SecurityBlackList>>,
    pub blacklist_ip_match: DashMap<String, Vec<SecurityBlackList>>,
}

impl Default for SecurityMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityMetadata {
    pub fn new() -> Self {
        SecurityMetadata {
            authn_list: DashMap::with_capacity(2),

            // user
            user_info: DashMap::with_capacity(2),

            // blacklist
            blacklist_user: DashMap::with_capacity(2),
            blacklist_client_id: DashMap::with_capacity(2),
            blacklist_ip: DashMap::with_capacity(2),
            blacklist_user_match: DashMap::with_capacity(2),
            blacklist_client_id_match: DashMap::with_capacity(2),
            blacklist_ip_match: DashMap::with_capacity(2),

            // user
            acl_user: DashMap::with_capacity(2),
            acl_client_id: DashMap::with_capacity(2),
        }
    }

    // user
    pub fn add_user(&self, user: SecurityUser) {
        self.user_info
            .entry(user.tenant.clone())
            .or_insert_with(DashMap::new)
            .insert(user.username.clone(), user);
    }

    pub fn del_user(&self, tenant: &str, username: &str) {
        if let Some(tenant_map) = self.user_info.get(tenant) {
            tenant_map.remove(username);
        }
    }

    // ACL
    pub fn parse_mqtt_acl(&self, acl: SecurityAcl) {
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

    pub fn remove_mqtt_acl(&self, acl: SecurityAcl) {
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

    pub fn get_all_acl(&self) -> Vec<SecurityAcl> {
        let mut data: Vec<SecurityAcl> = Vec::new();
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

    pub fn get_acl_by_tenant(&self, tenant: &str) -> Vec<SecurityAcl> {
        let mut data: Vec<SecurityAcl> = Vec::new();
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
    pub fn parse_mqtt_blacklist(&self, blacklist: SecurityBlackList) {
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

    pub fn remove_mqtt_blacklist(&self, blacklist: SecurityBlackList) {
        match blacklist.blacklist_type {
            MqttAclBlackListType::ClientId => {
                if let Some(tenant_map) = self.blacklist_client_id.get(&blacklist.tenant) {
                    tenant_map.retain(|_, v| v.name != blacklist.name);
                }
            }
            MqttAclBlackListType::User => {
                if let Some(tenant_map) = self.blacklist_user.get(&blacklist.tenant) {
                    tenant_map.retain(|_, v| v.name != blacklist.name);
                }
            }
            MqttAclBlackListType::Ip => {
                if let Some(tenant_map) = self.blacklist_ip.get(&blacklist.tenant) {
                    tenant_map.retain(|_, v| v.name != blacklist.name);
                }
            }
            MqttAclBlackListType::ClientIdMatch => {
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_client_id_match.get_mut(&blacklist.tenant) {
                    data.retain(|item| item.name != blacklist.name);
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_client_id_match.remove(&blacklist.tenant);
                }
            }
            MqttAclBlackListType::UserMatch => {
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_user_match.get_mut(&blacklist.tenant) {
                    data.retain(|item| item.name != blacklist.name);
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_user_match.remove(&blacklist.tenant);
                }
            }
            MqttAclBlackListType::IPCIDR => {
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_ip_match.get_mut(&blacklist.tenant) {
                    data.retain(|item| item.name != blacklist.name);
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_ip_match.remove(&blacklist.tenant);
                }
            }
        }
    }

    /// Returns all wildcard user blacklist entries across all tenants.
    pub fn get_blacklist_user_match(&self) -> Vec<SecurityBlackList> {
        self.blacklist_user_match
            .iter()
            .flat_map(|entry| entry.value().clone())
            .collect()
    }

    /// Returns all wildcard client-id blacklist entries across all tenants.
    pub fn get_blacklist_client_id_match(&self) -> Vec<SecurityBlackList> {
        self.blacklist_client_id_match
            .iter()
            .flat_map(|entry| entry.value().clone())
            .collect()
    }

    /// Returns all CIDR/wildcard IP blacklist entries across all tenants.
    pub fn get_blacklist_ip_match(&self) -> Vec<SecurityBlackList> {
        self.blacklist_ip_match
            .iter()
            .flat_map(|entry| entry.value().clone())
            .collect()
    }

    pub fn get_all_blacklist(&self) -> Vec<SecurityBlackList> {
        let mut data: Vec<SecurityBlackList> = Vec::new();
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

    pub fn get_blacklist_by_tenant(&self, tenant: &str) -> Vec<SecurityBlackList> {
        let mut data: Vec<SecurityBlackList> = Vec::new();
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

    // Authn
    pub fn add_authn(&self, auth: AuthnConfig) {
        self.authn_list.insert(auth.uid.clone(), auth);
    }

    pub fn get_authn(&self) -> Vec<(String, AuthnConfig)> {
        let mut authn_list: Vec<(String, AuthnConfig)> = self
            .authn_list
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        authn_list.sort_by(|a, b| b.1.create_at.cmp(&a.1.create_at));
        authn_list
    }

    pub fn remove_authn(&self, uid: &str) {
        self.authn_list.remove(uid);
    }
}

#[cfg(test)]
mod test {
    use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
    use common_base::enum_type::mqtt::acl::mqtt_acl_blacklist_type::MqttAclBlackListType;
    use common_base::enum_type::mqtt::acl::mqtt_acl_permission::MqttAclPermission;
    use common_base::enum_type::mqtt::acl::mqtt_acl_resource_type::MqttAclResourceType;
    use common_base::tools::now_second;
    use metadata_struct::auth::acl::SecurityAcl;
    use metadata_struct::auth::blacklist::SecurityBlackList;
    use metadata_struct::tenant::DEFAULT_TENANT;

    use crate::metadata::SecurityMetadata;

    const TENANT: &str = DEFAULT_TENANT;

    #[tokio::test]
    pub async fn parse_mqtt_acl_test() {
        let acl_metadata = SecurityMetadata::new();
        // Test ClientId ACL
        let client_id_acl = SecurityAcl {
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
        let user_acl = SecurityAcl {
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
        let client_id_acl2 = SecurityAcl {
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
        let user_acl2 = SecurityAcl {
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
        acl_metadata.remove_mqtt_acl(SecurityAcl {
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
        let acl_metadata = SecurityMetadata::new();

        // Test ClientId blacklist
        let client_id_blacklist = SecurityBlackList {
            name: "bl-client-id".to_string(),
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
        let user_blacklist = SecurityBlackList {
            name: "bl-user".to_string(),
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
        let ip_blacklist = SecurityBlackList {
            name: "bl-ip".to_string(),
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
        let client_id_match_blacklist = SecurityBlackList {
            name: "bl-client-id-match".to_string(),
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
        let user_match_blacklist = SecurityBlackList {
            name: "bl-user-match".to_string(),
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
        let ip_cidr_blacklist = SecurityBlackList {
            name: "bl-ip-cidr".to_string(),
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
        let another_client_id_match_blacklist = SecurityBlackList {
            name: "bl-client-id-match-2".to_string(),
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
        acl_metadata.remove_mqtt_blacklist(SecurityBlackList {
            name: "bl-client-id-match".to_string(),
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
        let acl_metadata = SecurityMetadata::new();
        acl_metadata.parse_mqtt_acl(SecurityAcl {
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
        acl_metadata.parse_mqtt_acl(SecurityAcl {
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
