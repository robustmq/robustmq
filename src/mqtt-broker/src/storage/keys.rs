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

use crate::handler::{
    flapping_detect::BanLog, sub_slow::SlowSubscribeData, system_alarm::SystemAlarmEventMessage,
};

pub fn system_event_key(alarm: &SystemAlarmEventMessage) -> String {
    prefix_key(format!(
        "/system_alarm/{}/{}",
        alarm.name, alarm.create_time
    ))
}

pub fn system_event_prefix_key() -> String {
    prefix_key("/system_alarm/".to_string())
}

pub fn ban_log_key(log: &BanLog) -> String {
    prefix_key(format!(
        "/ban_log/{}/{}/{}",
        log.ban_type, log.resource_name, log.create_time
    ))
}

pub fn ban_log_prefix_key() -> String {
    prefix_key("/ban_log/".to_string())
}

pub fn slow_sub_log_key(data: &SlowSubscribeData) -> String {
    prefix_key(format!(
        "/slow_sub_log/{}/{}",
        data.client_id, data.topic_name
    ))
}

pub fn slow_sub_log_prefix_key() -> String {
    prefix_key("/slow_sub_log/".to_string())
}

fn prefix_key(key: String) -> String {
    format!("/broker/mqtt/{key}")
}
