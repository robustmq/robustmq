use metadata_struct::acl::{mqtt_acl::MQTTAcl, mqtt_blacklist::MQTTAclBlackList};

#[derive(Clone)]
pub struct AclMetadata {}

impl AclMetadata {
    pub fn new() -> Self {
        return AclMetadata {};
    }

    pub fn parse_mqtt_acl(&self, acl: MQTTAcl) {}

    pub fn parse_mqtt_blacklist(&self, blacklist: MQTTAclBlackList) {}

    
}

#[cfg(test)]
mod test {
    #[tokio::test]
    pub async fn parse_mqtt_acl_test() {}

    #[tokio::test]
    pub async fn parse_mqtt_blacklist() {}
}
