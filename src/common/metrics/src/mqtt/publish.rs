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

use crate::{gauge_metric_inc, register_gauge_metric};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct MessageLabel {}

register_gauge_metric!(
    MQTT_MESSAGES_DELAYED,
    "mqtt_messages_delayed",
    "Number of delayed publish messages stored",
    MessageLabel
);

register_gauge_metric!(
    MQTT_MESSAGES_RECEIVED,
    "mqtt_messages_received",
    "Number of messages received from clients",
    MessageLabel
);

register_gauge_metric!(
    MQTT_MESSAGES_SENT,
    "mqtt_messages_sent",
    "Number of messages sent to clients",
    MessageLabel
);

pub fn record_mqtt_messages_delayed_inc() {
    let label = MessageLabel {};
    gauge_metric_inc!(MQTT_MESSAGES_DELAYED, label);
}

pub fn record_mqtt_messages_received_inc() {
    let label = MessageLabel {};
    gauge_metric_inc!(MQTT_MESSAGES_RECEIVED, label);
}

pub fn record_mqtt_messages_sent_inc() {
    let label = MessageLabel {};
    gauge_metric_inc!(MQTT_MESSAGES_SENT, label);
}
