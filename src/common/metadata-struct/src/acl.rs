// Copyright 2023 RobustMQ Team
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

use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

pub const WILDCARD_RESOURCE: &str = "*";

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct CommonAcl {
    pub resource_type: AclResourceType,
    pub resource_name: String,
    pub pattern_type: AclPatternType,
    pub principal: String,
    pub principal_type: AclPrincipalType,
    pub acl_operation: AclOperation,
    pub acl_permission_type: AclPermissionType,
    pub host: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum AclResourceType {
    Any,
    Topic,
    Group,
    Cluster,
    User,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum AclPrincipalType {
    User,
    IP,
    ClientId,
}

impl Display for AclPrincipalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AclPrincipalType::User => write!(f, "User"),
            AclPrincipalType::IP => write!(f, "Ip"),
            AclPrincipalType::ClientId => write!(f, "ClientId"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum AclPatternType {
    Any,
    Match,
    Literal,
    Prefixed,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum AclOperation {
    Any,
    Read,
    Write,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum AclPermissionType {
    Any,
    Deny,
    Allow,
}
