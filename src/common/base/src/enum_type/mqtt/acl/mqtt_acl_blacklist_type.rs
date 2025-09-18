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

use clap::builder::PossibleValue;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

use crate::error::common::CommonError;

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum MqttAclBlackListType {
    ClientId,
    User,
    Ip,
    ClientIdMatch,
    UserMatch,
    IPCIDR,
}

impl FromStr for MqttAclBlackListType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, true) {
                return Ok(*variant);
            }
        }
        Err(format!("invalid variant: {s}"))
    }
}

impl ValueEnum for MqttAclBlackListType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::ClientId,
            Self::User,
            Self::Ip,
            Self::ClientIdMatch,
            Self::UserMatch,
            Self::IPCIDR,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            MqttAclBlackListType::ClientId => PossibleValue::new("ClientId"),
            MqttAclBlackListType::User => PossibleValue::new("User"),
            MqttAclBlackListType::Ip => PossibleValue::new("Ip"),
            MqttAclBlackListType::ClientIdMatch => PossibleValue::new("ClientIdMatch"),
            MqttAclBlackListType::UserMatch => PossibleValue::new("UserMatch"),
            MqttAclBlackListType::IPCIDR => PossibleValue::new("IPCIDR"),
        })
    }
}

pub fn get_blacklist_type_by_str(
    blacklist_type: &str,
) -> Result<MqttAclBlackListType, CommonError> {
    let blacklist_type = match blacklist_type {
        "ClientId" => MqttAclBlackListType::ClientId,
        "User" => MqttAclBlackListType::User,
        "Ip" => MqttAclBlackListType::Ip,
        "ClientIdMatch" => MqttAclBlackListType::ClientIdMatch,
        "UserMatch" => MqttAclBlackListType::UserMatch,
        "IPCIDR" => MqttAclBlackListType::IPCIDR,
        _ => {
            return Err(CommonError::CommonError(format!(
                "Failed BlackList Type: {blacklist_type}",
            )))
        }
    };
    Ok(blacklist_type)
}

impl fmt::Display for MqttAclBlackListType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}
