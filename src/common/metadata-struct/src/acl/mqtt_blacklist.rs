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

use std::fmt;

use common_base::error::common::CommonError;
use protocol::broker_mqtt::broker_mqtt_admin::{BlacklistRaw, BlacklistType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct MqttAclBlackList {
    pub blacklist_type: MqttAclBlackListType,
    pub resource_name: String,
    pub end_time: u64,
    pub desc: String,
}

impl MqttAclBlackList {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        Ok(serde_json::from_slice(data)?)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum MqttAclBlackListType {
    ClientId,
    User,
    Ip,
    ClientIdMatch,
    UserMatch,
    IPCIDR,
}

impl fmt::Display for MqttAclBlackListType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MqttAclBlackListType::ClientId => "ClientId",
                MqttAclBlackListType::User => "User",
                MqttAclBlackListType::Ip => "Ip",
                MqttAclBlackListType::ClientIdMatch => "ClientIdMatch",
                MqttAclBlackListType::UserMatch => "UserMatch",
                MqttAclBlackListType::IPCIDR => "IPCIDR",
            }
        )
    }
}

impl From<MqttAclBlackListType> for BlacklistType {
    fn from(type_enum: MqttAclBlackListType) -> Self {
        match type_enum {
            MqttAclBlackListType::ClientId => Self::ClientId,
            MqttAclBlackListType::User => Self::Username,
            MqttAclBlackListType::Ip => Self::IpAddress,
            MqttAclBlackListType::ClientIdMatch => Self::ClientIdMatch,
            MqttAclBlackListType::UserMatch => Self::Username,
            MqttAclBlackListType::IPCIDR => Self::IpCidr,
        }
    }
}

impl From<MqttAclBlackList> for BlacklistRaw {
    fn from(blacklist: MqttAclBlackList) -> Self {
        let blacklist_type_as_enum: BlacklistType = blacklist.blacklist_type.into();
        Self {
            blacklist_type: blacklist_type_as_enum as i32,
            resource_name: blacklist.resource_name,
            end_time: blacklist.end_time,
            desc: blacklist.desc,
        }
    }
}
