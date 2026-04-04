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

use clap::{builder::PossibleValue, ValueEnum};
use common_base::error::common::CommonError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct SecurityAcl {
    pub name: String,
    pub desc: String,
    pub tenant: String,
    pub resource_type: EnumAclResourceType,
    pub resource_name: String,
    pub topic: String,
    pub ip: String,
    pub action: EnumAclAction,
    pub permission: EnumAclPermission,
}

impl SecurityAcl {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        Ok(serde_json::from_slice(data)?)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum EnumAclPermission {
    Allow,
    Deny,
}

impl FromStr for EnumAclPermission {
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

impl ValueEnum for EnumAclPermission {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Allow, Self::Deny]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            EnumAclPermission::Allow => PossibleValue::new("Allow"),
            EnumAclPermission::Deny => PossibleValue::new("Deny"),
        })
    }
}

impl fmt::Display for EnumAclPermission {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum EnumAclResourceType {
    ClientId,
    User,
}

impl FromStr for EnumAclResourceType {
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

impl ValueEnum for EnumAclResourceType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::ClientId, Self::User]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            EnumAclResourceType::ClientId => PossibleValue::new("ClientId"),
            EnumAclResourceType::User => PossibleValue::new("User"),
        })
    }
}

impl fmt::Display for EnumAclResourceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum EnumAclAction {
    All,
    Subscribe,
    Publish,
    PubSub,
    Retain,
    Qos,
}

impl FromStr for EnumAclAction {
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

impl ValueEnum for EnumAclAction {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::All,
            Self::Subscribe,
            Self::Publish,
            Self::PubSub,
            Self::Retain,
            Self::Qos,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            EnumAclAction::All => PossibleValue::new("All"),
            EnumAclAction::Subscribe => PossibleValue::new("Subscribe"),
            EnumAclAction::Publish => PossibleValue::new("Publish"),
            EnumAclAction::PubSub => PossibleValue::new("PubSub"),
            EnumAclAction::Retain => PossibleValue::new("Retain"),
            EnumAclAction::Qos => PossibleValue::new("Qos"),
        })
    }
}

impl fmt::Display for EnumAclAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}
