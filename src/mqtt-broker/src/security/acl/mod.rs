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

#[allow(dead_code)]
pub struct Acl {
    pub allow: AclAllow,
    pub ip_addr: String,
    pub username: String,
    pub client_id: String,
    pub access: AclAccess,
    pub topic: String,
}

#[allow(dead_code)]
pub enum AclAllow {
    Deny,
    Allow,
}

#[allow(dead_code)]
pub enum AclAccess {
    Subscribe,
    Publish,
    PubSub,
}


pub fn authentication_acl() -> bool {
    return false;
}