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

use crate::auth::acl::is_acl_deny;
use crate::login::super_user::is_super_user;
use crate::manager::SecurityManager;
use metadata_struct::auth::acl::EnumAclAction;
use protocol::mqtt::common::QoS;
use std::sync::Arc;

pub mod acl;
pub mod blacklist;
pub mod common;

pub fn is_allow_acl(
    security_manager: &Arc<SecurityManager>,
    topic_name: &str,
    tenant: &str,
    user: &str,
    source_ip_addr: &str,
    client_id: &str,
    action: EnumAclAction,
    retain: bool,
    _: QoS,
) -> bool {
    // check super user
    if is_super_user(security_manager, user) {
        return true;
    }

    // check acl
    if is_acl_deny(
        security_manager,
        topic_name,
        tenant,
        user,
        source_ip_addr,
        client_id,
        action,
    ) {
        return false;
    }

    // check retain acl
    if retain
        && is_acl_deny(
            security_manager,
            topic_name,
            tenant,
            user,
            source_ip_addr,
            client_id,
            EnumAclAction::Retain,
        )
    {
        return false;
    }

    true
}
