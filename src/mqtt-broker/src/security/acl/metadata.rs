use dashmap::DashMap;
use metadata_struct::acl::{
    mqtt_acl::{MQTTAcl, MQTTAclResourceType},
    mqtt_blacklist::{MQTTAclBlackList, MQTTAclBlackListType},
};

#[derive(Clone)]
pub struct AclMetadata {
    // blacklist
    pub blacklist_user: DashMap<String, MQTTAclBlackList>,
    pub blacklist_client_id: DashMap<String, MQTTAclBlackList>,
    pub blacklist_ip: DashMap<String, MQTTAclBlackList>,
    pub blacklist_user_match: DashMap<String, Vec<MQTTAclBlackList>>,
    pub blacklist_client_id_match: DashMap<String, Vec<MQTTAclBlackList>>,
    pub blacklist_ip_match: DashMap<String, Vec<MQTTAclBlackList>>,

    // acl
    pub acl_user: DashMap<String, Vec<MQTTAcl>>,
    pub acl_client_id: DashMap<String, Vec<MQTTAcl>>,
}

impl AclMetadata {
    pub fn new() -> Self {
        return AclMetadata {
            blacklist_user: DashMap::with_capacity(2),
            blacklist_client_id: DashMap::with_capacity(2),
            blacklist_ip: DashMap::with_capacity(2),
            blacklist_user_match: DashMap::with_capacity(2),
            blacklist_client_id_match: DashMap::with_capacity(2),
            blacklist_ip_match: DashMap::with_capacity(2),

            acl_user: DashMap::with_capacity(2),
            acl_client_id: DashMap::with_capacity(2),
        };
    }

    pub fn parse_mqtt_acl(&self, acl: MQTTAcl) {
        match acl.resource_type {
            MQTTAclResourceType::ClientId => {
                if let Some(mut raw) = self.acl_client_id.get_mut(&acl.resource_name) {
                    raw.push(acl);
                } else {
                    self.acl_client_id
                        .insert(acl.resource_name.clone(), vec![acl]);
                }
            }
            MQTTAclResourceType::User => {
                if let Some(mut raw) = self.acl_user.get_mut(&acl.resource_name) {
                    raw.push(acl);
                } else {
                    self.acl_user.insert(acl.resource_name.clone(), vec![acl]);
                }
            }
        }
    }

    pub fn parse_mqtt_blacklist(&self, blacklist: MQTTAclBlackList) {
        match blacklist.blacklist_type {
            MQTTAclBlackListType::ClientId => {
                self.blacklist_client_id
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            MQTTAclBlackListType::User => {
                self.blacklist_user
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            MQTTAclBlackListType::Ip => {
                self.blacklist_ip
                    .insert(blacklist.resource_name.clone(), blacklist);
            }
            MQTTAclBlackListType::ClientIdMatch => {
                let key = self.get_client_id_match_key();
                if let Some(mut data) = self.blacklist_client_id_match.get_mut(&key) {
                    data.push(blacklist)
                } else {
                    self.blacklist_client_id_match.insert(key, vec![blacklist]);
                }
            }
            MQTTAclBlackListType::UserMatch => {
                let key = self.get_user_match_key();
                if let Some(mut data) = self.blacklist_user_match.get_mut(&key) {
                    data.push(blacklist)
                } else {
                    self.blacklist_user_match.insert(key, vec![blacklist]);
                }
            }
            MQTTAclBlackListType::IPCIDR => {
                let key = self.get_ip_cidr_key();
                if let Some(mut data) = self.blacklist_ip_match.get_mut(&key) {
                    data.push(blacklist)
                } else {
                    self.blacklist_ip_match.insert(key, vec![blacklist]);
                }
            }
        }
    }

    pub fn get_blacklist_user_match(&self) -> Option<Vec<MQTTAclBlackList>> {
        let key = self.get_user_match_key();
        if let Some(data) = self.blacklist_user_match.get(&key) {
            return Some(data.clone());
        }
        return None;
    }

    pub fn get_blacklist_client_id_match(&self) -> Option<Vec<MQTTAclBlackList>> {
        let key = self.get_client_id_match_key();
        if let Some(data) = self.blacklist_client_id_match.get(&key) {
            return Some(data.clone());
        }
        return None;
    }

    pub fn get_blacklist_ip_match(&self) -> Option<Vec<MQTTAclBlackList>> {
        let key = self.get_ip_cidr_key();
        if let Some(data) = self.blacklist_ip_match.get(&key) {
            return Some(data.clone());
        }
        return None;
    }

    fn get_client_id_match_key(&self) -> String {
        return "ClientIdMatch".to_string();
    }

    fn get_user_match_key(&self) -> String {
        return "UserMatch".to_string();
    }

    fn get_ip_cidr_key(&self) -> String {
        return "IPCIDR".to_string();
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    pub async fn parse_mqtt_acl_test() {}

    #[tokio::test]
    pub async fn parse_mqtt_blacklist() {}
}
