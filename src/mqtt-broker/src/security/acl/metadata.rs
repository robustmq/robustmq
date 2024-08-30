use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::acl::{
    mqtt_acl::{MQTTAcl, MQTTAclResourceType},
    mqtt_blacklist::{MQTTAclBlackList, MQTTAclBlackListType},
};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AclMetadata {
    // blacklist
    pub blacklist_user: DashMap<String, MQTTAclBlackList>,
    pub blacklist_client_id: DashMap<String, MQTTAclBlackList>,
    pub blacklist_ip: DashMap<String, MQTTAclBlackList>,
    pub blacklist_user_match: Arc<RwLock<Vec<MQTTAclBlackList>>>,
    pub blacklist_client_id_match: Arc<RwLock<Vec<MQTTAclBlackList>>>,
    pub blacklist_ip_match: Arc<RwLock<Vec<MQTTAclBlackList>>>,

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
            blacklist_user_match: Arc::new(RwLock::new(Vec::new())),
            blacklist_client_id_match: Arc::new(RwLock::new(Vec::new())),
            blacklist_ip_match: Arc::new(RwLock::new(Vec::new())),

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

    pub fn parse_mqtt_blacklist(&self, blacklist: MQTTAclBlackList) -> Result<(), CommonError> {
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
                let mut data = match self.blacklist_client_id_match.write() {
                    Ok(da) => da,
                    Err(e) => {
                        return Err(CommonError::CommmonError(e.to_string()));
                    }
                };

                data.push(blacklist)
            }
            MQTTAclBlackListType::UserMatch => {
                let mut data = match self.blacklist_user_match.write() {
                    Ok(da) => da,
                    Err(e) => {
                        return Err(CommonError::CommmonError(e.to_string()));
                    }
                };

                data.push(blacklist)
            }
            MQTTAclBlackListType::IPCIDR => {
                let mut data = match self.blacklist_ip_match.write() {
                    Ok(da) => da,
                    Err(e) => {
                        return Err(CommonError::CommmonError(e.to_string()));
                    }
                };

                data.push(blacklist)
            }
        }
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    pub async fn parse_mqtt_acl_test() {}

    #[tokio::test]
    pub async fn parse_mqtt_blacklist() {}
}
