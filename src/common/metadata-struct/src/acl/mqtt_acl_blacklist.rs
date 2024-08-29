use std::fmt;

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
    IPCIDR,
}

impl fmt::Display for MQTTAclBlackListType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MQTTAclBlackListType::ClientId => "ClientId",
                MQTTAclBlackListType::User => "User",
                MQTTAclBlackListType::Ip => "Ip",
                MQTTAclBlackListType::ClientIdMatch => "ClientIdMatch",
                MQTTAclBlackListType::UserMatch => "UserMatch",
                MQTTAclBlackListType::IPCIDR => "IPCIDR",
            }
        )
    }
}
