use std::fmt;

use common_base::error::common::CommonError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct MQTTAclBlackList {
    pub blacklist_type: MQTTAclBlackListType,
    pub resource_name: String,
    pub end_time: u64,
    pub desc: String,
}

impl MQTTAclBlackList {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        return Ok(serde_json::to_vec(&self)?);
    }
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
