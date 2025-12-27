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

use super::PREFIX_BROKER;

pub fn system_event_key(alarm_name: &str, create_time: i64) -> String {
    format!(
        "{}system_alarm/{}/{}",
        PREFIX_BROKER, alarm_name, create_time
    )
}

pub fn system_event_prefix_key() -> String {
    format!("{}system_alarm/", PREFIX_BROKER)
}

pub fn ban_log_key(ban_type: &str, resource_name: &str, create_time: i64) -> String {
    format!(
        "{}ban_log/{}/{}/{}",
        PREFIX_BROKER, ban_type, resource_name, create_time
    )
}

pub fn ban_log_prefix_key() -> String {
    format!("{}ban_log/", PREFIX_BROKER)
}

pub fn slow_sub_log_key(client_id: &str, topic_name: &str) -> String {
    format!("{}slow_sub_log/{}/{}", PREFIX_BROKER, client_id, topic_name)
}

pub fn slow_sub_log_prefix_key() -> String {
    format!("{}slow_sub_log/", PREFIX_BROKER)
}
