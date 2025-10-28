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

use crate::{counter_metric_inc, register_counter_metric};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct AuthLabel {}

register_counter_metric!(
    MQTT_AUTH_SUCCESS,
    "mqtt_auth_success",
    "Number of successful MQTT authentications",
    AuthLabel
);

register_counter_metric!(
    MQTT_AUTH_FAILED,
    "mqtt_auth_failed",
    "Number of failed MQTT authentications",
    AuthLabel
);

register_counter_metric!(
    MQTT_ACL_SUCCESS,
    "mqtt_acl_success",
    "Number of successful MQTT ACL checks",
    AuthLabel
);

register_counter_metric!(
    MQTT_ACL_FAILED,
    "mqtt_acl_failed",
    "Number of failed MQTT ACL checks",
    AuthLabel
);

register_counter_metric!(
    MQTT_BLACKLIST_BLOCKED,
    "mqtt_blacklist_blocked",
    "Number of MQTT connections blocked by blacklist",
    AuthLabel
);

pub fn record_mqtt_auth_success() {
    let label = AuthLabel {};
    counter_metric_inc!(MQTT_AUTH_SUCCESS, label);
}

pub fn record_mqtt_auth_failed() {
    let label = AuthLabel {};
    counter_metric_inc!(MQTT_AUTH_FAILED, label);
}

pub fn record_mqtt_acl_success() {
    let label = AuthLabel {};
    counter_metric_inc!(MQTT_ACL_SUCCESS, label);
}

pub fn record_mqtt_acl_failed() {
    let label = AuthLabel {};
    counter_metric_inc!(MQTT_ACL_FAILED, label);
}

pub fn record_mqtt_blacklist_blocked() {
    let label = AuthLabel {};
    counter_metric_inc!(MQTT_BLACKLIST_BLOCKED, label);
}
