use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct MQTTAclBlackList {
    pub blacklist_type: MQTTAclBlackListType,
    pub resource_name: String,
    pub end_time: u64,
    pub desc: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum MQTTAclBlackListType {
    ClientId,
    User,
    Ip,
    ClientIdMatch,
    UserMatch,
    IpCIDR
}
