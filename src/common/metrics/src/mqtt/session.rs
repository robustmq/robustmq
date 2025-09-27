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
pub struct NetworkLabel {}

register_counter_metric!(
    MQTT_SESSION_CREATED,
    "mqtt_session_created",
    "Number of MQTT sessions created",
    NetworkLabel
);

register_counter_metric!(
    MQTT_SESSION_DELETED,
    "mqtt_session_deleted",
    "Number of MQTT sessions deleted",
    NetworkLabel
);

pub fn record_mqtt_session_created() {
    let label = NetworkLabel {};
    counter_metric_inc!(MQTT_SESSION_CREATED, label);
}

pub fn record_mqtt_session_deleted() {
    let label = NetworkLabel {};
    counter_metric_inc!(MQTT_SESSION_DELETED, label);
}
