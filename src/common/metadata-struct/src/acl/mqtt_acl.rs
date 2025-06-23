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
use protocol::broker_mqtt::broker_mqtt_admin::{AclAction, AclPermission, AclRaw, AclResourceType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct MqttAcl {
    pub resource_type: MqttAclResourceType,
    pub resource_name: String,
    pub topic: String,
    pub ip: String,
    pub action: MqttAclAction,
    pub permission: MqttAclPermission,
}

impl MqttAcl {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        Ok(serde_json::from_slice(data)?)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum MqttAclResourceType {
    ClientId,
    User,
}

impl fmt::Display for MqttAclResourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MqttAclResourceType::ClientId => "ClientId",
                MqttAclResourceType::User => "User",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum MqttAclAction {
    All,
    Subscribe,
    Publish,
    PubSub,
    Retain,
    Qos,
}

impl fmt::Display for MqttAclAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MqttAclAction::All => "All",
                MqttAclAction::Subscribe => "Subscribe",
                MqttAclAction::Publish => "Publish",
                MqttAclAction::PubSub => "PubSub",
                MqttAclAction::Retain => "Retain",
                MqttAclAction::Qos => "Qos",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum MqttAclPermission {
    Allow,
    Deny,
}

impl fmt::Display for MqttAclPermission {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MqttAclPermission::Allow => "Allow",
                MqttAclPermission::Deny => "Deny",
            }
        )
    }
}

impl From<MqttAclResourceType> for AclResourceType {
    fn from(type_enum: MqttAclResourceType) -> Self {
        match type_enum {
            MqttAclResourceType::ClientId => Self::ClientId,
            MqttAclResourceType::User => Self::User,
        }
    }
}

impl From<MqttAclAction> for AclAction {
    fn from(action: MqttAclAction) -> Self {
        match action {
            MqttAclAction::All => Self::All,
            MqttAclAction::Subscribe => Self::Subscribe,
            MqttAclAction::Publish => Self::Publish,
            MqttAclAction::PubSub => Self::PubSub,
            MqttAclAction::Retain => Self::Retain,
            MqttAclAction::Qos => Self::Qos,
        }
    }
}

impl From<MqttAclPermission> for AclPermission {
    fn from(permission: MqttAclPermission) -> Self {
        match permission {
            MqttAclPermission::Allow => Self::Allow,
            MqttAclPermission::Deny => Self::Deny,
        }
    }
}

impl From<MqttAcl> for AclRaw {
    fn from(acl: MqttAcl) -> Self {
        let acl_resource_type: AclResourceType = acl.resource_type.into();
        let acl_action: AclAction = acl.action.into();
        let acl_permission: AclPermission = acl.permission.into();
        Self {
            resource_type: acl_resource_type as i32,
            resource_name: acl.resource_name,
            topic: acl.topic,
            ip: acl.ip,
            action: acl_action as i32,
            permission: acl_permission as i32,
        }
    }
}
