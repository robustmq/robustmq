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
use metadata_struct::auth::acl::{EnumAclResourceType, SecurityAcl};
use metadata_struct::auth::blacklist::{EnumBlackListType, SecurityBlackList};
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

            // acl
            acl_user: DashMap::with_capacity(2),
            acl_client_id: DashMap::with_capacity(2),
        }
    }

    // user
    pub fn add_user(&self, user: SecurityUser) {
        self.user_info
            .entry(user.tenant.clone())
            .or_default()
            .insert(user.username.clone(), user);
    }

    pub fn del_user(&self, tenant: &str, username: &str) {
        if let Some(tenant_map) = self.user_info.get(tenant) {
            tenant_map.remove(username);
        }
    }

    // ACL
    pub fn add_acl(&self, acl: SecurityAcl) {
        self.parse_mqtt_acl(acl);
    }

    pub fn remove_acl(&self, acl: SecurityAcl) {
        self.remove_mqtt_acl(acl);
    }

    pub fn parse_mqtt_acl(&self, acl: SecurityAcl) {
        let map = match acl.resource_type {
            EnumAclResourceType::ClientId => &self.acl_client_id,
            EnumAclResourceType::User => &self.acl_user,
        };
        map.entry(acl.tenant.clone())
            .or_default()
            .entry(acl.resource_name.clone())
            .or_default()
            .push(acl);
    }

    pub fn remove_mqtt_acl(&self, acl: SecurityAcl) {
        let map = match acl.resource_type {
            EnumAclResourceType::ClientId => &self.acl_client_id,
            EnumAclResourceType::User => &self.acl_user,
        };
        if let Some(tenant_map) = map.get(&acl.tenant) {
            if let Some(mut list) = tenant_map.get_mut(&acl.resource_name) {
                list.retain(|item| item.name != acl.name);
                if list.is_empty() {
                    drop(list);
                    tenant_map.remove(&acl.resource_name);
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
    pub fn add_blacklist(&self, blacklist: SecurityBlackList) {
        self.parse_mqtt_blacklist(blacklist);
    }

    pub fn remove_blacklist(&self, blacklist: SecurityBlackList) {
        self.remove_mqtt_blacklist(blacklist);
    }

    fn parse_mqtt_blacklist(&self, blacklist: SecurityBlackList) {
        match blacklist.blacklist_type {
            EnumBlackListType::ClientId => {
                self.blacklist_client_id
                    .entry(blacklist.tenant.clone())
                    .or_default()
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            EnumBlackListType::User => {
                self.blacklist_user
                    .entry(blacklist.tenant.clone())
                    .or_default()
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            EnumBlackListType::Ip => {
                self.blacklist_ip
                    .entry(blacklist.tenant.clone())
                    .or_default()
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            EnumBlackListType::ClientIdMatch => {
                self.blacklist_client_id_match
                    .entry(blacklist.tenant.clone())
                    .or_default()
                    .push(blacklist);
            }
            EnumBlackListType::UserMatch => {
                self.blacklist_user_match
                    .entry(blacklist.tenant.clone())
                    .or_default()
                    .push(blacklist);
            }
            EnumBlackListType::IPCIDR => {
                self.blacklist_ip_match
                    .entry(blacklist.tenant.clone())
                    .or_default()
                    .push(blacklist);
            }
        }
    }

    pub fn remove_mqtt_blacklist(&self, blacklist: SecurityBlackList) {
        match blacklist.blacklist_type {
            EnumBlackListType::ClientId => {
                if let Some(tenant_map) = self.blacklist_client_id.get(&blacklist.tenant) {
                    tenant_map.retain(|_, v| v.name != blacklist.name);
                }
            }
            EnumBlackListType::User => {
                if let Some(tenant_map) = self.blacklist_user.get(&blacklist.tenant) {
                    tenant_map.retain(|_, v| v.name != blacklist.name);
                }
            }
            EnumBlackListType::Ip => {
                if let Some(tenant_map) = self.blacklist_ip.get(&blacklist.tenant) {
                    tenant_map.retain(|_, v| v.name != blacklist.name);
                }
            }
            EnumBlackListType::ClientIdMatch => {
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_client_id_match.get_mut(&blacklist.tenant) {
                    data.retain(|item| item.name != blacklist.name);
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_client_id_match.remove(&blacklist.tenant);
                }
            }
            EnumBlackListType::UserMatch => {
                let mut remove_key = false;
                if let Some(mut data) = self.blacklist_user_match.get_mut(&blacklist.tenant) {
                    data.retain(|item| item.name != blacklist.name);
                    remove_key = data.is_empty();
                }
                if remove_key {
                    self.blacklist_user_match.remove(&blacklist.tenant);
                }
            }
            EnumBlackListType::IPCIDR => {
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

    pub fn authn_list(&self) -> Vec<(String, AuthnConfig)> {
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
    use common_base::tools::now_second;
    use metadata_struct::auth::acl::{
        EnumAclAction, EnumAclPermission, EnumAclResourceType, SecurityAcl,
    };
    use metadata_struct::auth::blacklist::{EnumBlackListType, SecurityBlackList};
    use metadata_struct::tenant::DEFAULT_TENANT;

    use crate::metadata::SecurityMetadata;

    const TENANT: &str = DEFAULT_TENANT;

    #[tokio::test]
    pub async fn parse_mqtt_acl_test() {
        let acl_metadata = SecurityMetadata::new();
        let user_acl = SecurityAcl {
            name: "acl-user-1".to_string(),
            desc: String::new(),
            tenant: TENANT.to_string(),
            resource_type: EnumAclResourceType::User,
            resource_name: "test_user".to_string(),
            topic: "".to_string(),
            ip: "".to_string(),
            action: EnumAclAction::All,
            permission: EnumAclPermission::Allow,
        };
        acl_metadata.parse_mqtt_acl(user_acl.clone());
        acl_metadata.parse_mqtt_acl(SecurityAcl {
            name: "acl-user-2".to_string(),
            ..user_acl
        });
        assert_eq!(
            acl_metadata
                .acl_user
                .get(TENANT)
                .and_then(|m| m.get("test_user").map(|v| v.len()))
                .unwrap_or(0),
            2
        );

        acl_metadata.parse_mqtt_acl(SecurityAcl {
            name: "acl-client-1".to_string(),
            desc: String::new(),
            tenant: TENANT.to_string(),
            resource_type: EnumAclResourceType::ClientId,
            resource_name: "test_client".to_string(),
            topic: "".to_string(),
            ip: "".to_string(),
            action: EnumAclAction::All,
            permission: EnumAclPermission::Allow,
        });
        assert_eq!(acl_metadata.get_all_acl().len(), 3);

        acl_metadata.remove_mqtt_acl(SecurityAcl {
            name: "acl-user-1".to_string(),
            desc: String::new(),
            tenant: TENANT.to_string(),
            resource_type: EnumAclResourceType::User,
            resource_name: "test_user".to_string(),
            topic: "".to_string(),
            ip: "".to_string(),
            action: EnumAclAction::All,
            permission: EnumAclPermission::Allow,
        });
        assert_eq!(acl_metadata.get_all_acl().len(), 2);

        acl_metadata.remove_mqtt_acl(SecurityAcl {
            name: "acl-user-2".to_string(),
            desc: String::new(),
            tenant: TENANT.to_string(),
            resource_type: EnumAclResourceType::User,
            resource_name: "test_user".to_string(),
            topic: "".to_string(),
            ip: "".to_string(),
            action: EnumAclAction::All,
            permission: EnumAclPermission::Allow,
        });
        assert_eq!(acl_metadata.get_all_acl().len(), 1);
        let key_removed = acl_metadata
            .acl_user
            .get(TENANT)
            .map(|m| !m.contains_key("test_user"))
            .unwrap_or(true);
        assert!(key_removed);
    }

    #[tokio::test]
    pub async fn parse_mqtt_blacklist_test() {
        let acl_metadata = SecurityMetadata::new();
        let end_time = now_second() + 100;

        acl_metadata.parse_mqtt_blacklist(SecurityBlackList {
            name: "bl-client-id".to_string(),
            tenant: TENANT.to_string(),
            blacklist_type: EnumBlackListType::ClientId,
            resource_name: "test_client".to_string(),
            end_time,
            desc: "".to_string(),
        });
        acl_metadata.parse_mqtt_blacklist(SecurityBlackList {
            name: "bl-user".to_string(),
            tenant: TENANT.to_string(),
            blacklist_type: EnumBlackListType::User,
            resource_name: "test_user".to_string(),
            end_time,
            desc: "".to_string(),
        });
        acl_metadata.parse_mqtt_blacklist(SecurityBlackList {
            name: "bl-ip".to_string(),
            tenant: TENANT.to_string(),
            blacklist_type: EnumBlackListType::Ip,
            resource_name: "192.168.1.1".to_string(),
            end_time,
            desc: "".to_string(),
        });
        acl_metadata.parse_mqtt_blacklist(SecurityBlackList {
            name: "bl-client-id-match".to_string(),
            tenant: TENANT.to_string(),
            blacklist_type: EnumBlackListType::ClientIdMatch,
            resource_name: "test_client_*".to_string(),
            end_time,
            desc: "".to_string(),
        });
        acl_metadata.parse_mqtt_blacklist(SecurityBlackList {
            name: "bl-client-id-match-2".to_string(),
            tenant: TENANT.to_string(),
            blacklist_type: EnumBlackListType::ClientIdMatch,
            resource_name: "another_client_*".to_string(),
            end_time,
            desc: "".to_string(),
        });
        acl_metadata.parse_mqtt_blacklist(SecurityBlackList {
            name: "bl-user-match".to_string(),
            tenant: TENANT.to_string(),
            blacklist_type: EnumBlackListType::UserMatch,
            resource_name: "test_user_*".to_string(),
            end_time,
            desc: "".to_string(),
        });
        acl_metadata.parse_mqtt_blacklist(SecurityBlackList {
            name: "bl-ip-cidr".to_string(),
            tenant: TENANT.to_string(),
            blacklist_type: EnumBlackListType::IPCIDR,
            resource_name: "192.168.1.0/24".to_string(),
            end_time,
            desc: "".to_string(),
        });

        assert_eq!(acl_metadata.get_all_blacklist().len(), 7);
        assert_eq!(
            acl_metadata
                .blacklist_client_id_match
                .get(TENANT)
                .map(|v| v.len())
                .unwrap_or(0),
            2
        );

        acl_metadata.remove_mqtt_blacklist(SecurityBlackList {
            name: "bl-client-id-match".to_string(),
            tenant: TENANT.to_string(),
            blacklist_type: EnumBlackListType::ClientIdMatch,
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
}
