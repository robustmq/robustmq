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

pub const WILDCARD_RESOURCE: &str = "*";

pub struct CommonAcl {
    pub resource_type: AclResourceType,
    pub resource_name: String,
    pub pattern_type: AclPatternType,
    pub principal: String,
    pub acl_operation: AclOperation,
    pub acl_permission_type: AclPermissionType,
    pub host: String,
}

pub enum AclResourceType {
    Any,
    Topic,
    Group,
    Cluster,
    User,
}

pub enum AclPatternType {
    Any,
    Match,
    Literal,
    Prefixed,
}

pub enum AclOperation {
    Any,
    Read,
    Write,
}

pub enum AclPermissionType {
    Any,
    Deny,
    Allow,
}
