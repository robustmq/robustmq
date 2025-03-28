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

use paho_mqtt::Message;

#[derive(Debug, Clone, Default)]
pub struct ClientTestProperties {
    pub(crate) mqtt_version: u32,
    pub(crate) client_id: String,
    pub(crate) addr: String,
    pub(crate) ws: bool,
    pub(crate) ssl: bool,
    pub(crate) request_response: bool,
    pub(crate) user_name: String,
    pub(crate) password: String,
    pub(crate) will: Option<Message>,
    pub(crate) err_pwd: bool,
    pub(crate) conn_is_err: bool,
}
// pub mod acl_authorization_test;
pub mod common;
pub mod connect5_test;
pub mod connect_packet_size_test;
pub mod connect_test;
// mod flapping_detect_test;
pub mod auth_test;
pub mod keep_alive_test;
pub mod lastwill_message_test;
pub mod qos_test;
pub mod req_resp_test;
pub mod retain_message_test;
// pub mod trace_test;
// // pub mod share_sub_test;
pub mod sub_exclusive_test;
// pub mod sub_identifier_test;
// pub mod sub_options_test;
pub mod topic_alias_test;
// mod topic_rewrite_rule_test;
pub mod user_properties_test;
