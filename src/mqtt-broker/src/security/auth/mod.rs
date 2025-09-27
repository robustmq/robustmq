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

use crate::handler::cache::MQTTCacheManager;
use crate::security::auth::acl::is_acl_deny;
use crate::security::auth::super_user::is_super_user;
use common_base::enum_type::mqtt::acl::mqtt_acl_action::MqttAclAction;
use metadata_struct::mqtt::connection::MQTTConnection;
use protocol::mqtt::common::QoS;
use std::sync::Arc;

pub mod acl;
pub mod blacklist;
pub mod common;
pub mod metadata;
pub mod super_user;

pub fn is_allow_acl(
    cache_manager: &Arc<MQTTCacheManager>,
    connection: &MQTTConnection,
    topic_name: &str,
    action: MqttAclAction,
    retain: bool,
    _: QoS,
) -> bool {
    // check super user
    if is_super_user(cache_manager, &connection.login_user) {
        return true;
    }

    // check acl
    if is_acl_deny(cache_manager, connection, topic_name, action) {
        return false;
    }

    // check retain acl
    if retain && is_acl_deny(cache_manager, connection, topic_name, MqttAclAction::Retain) {
        return false;
    }

    true
}
