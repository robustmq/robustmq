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

use crate::{counter_metric_inc, counter_metric_inc_by, register_counter_metric};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct MessageLabel {
    topic: String,
}

register_counter_metric!(
    MQTT_MESSAGES_DELAYED,
    "mqtt_messages_delayed",
    "Total number of delayed publish messages",
    MessageLabel
);

register_counter_metric!(
    MQTT_MESSAGES_RECEIVED,
    "mqtt_messages_received",
    "Total number of messages received from clients",
    MessageLabel
);

register_counter_metric!(
    MQTT_MESSAGES_SENT,
    "mqtt_messages_sent",
    "Total number of messages sent to clients",
    MessageLabel
);

register_counter_metric!(
    MQTT_MESSAGE_BYTES_SENT,
    "mqtt_message_bytes_sent",
    "Total bytes of messages sent to clients",
    MessageLabel
);

register_counter_metric!(
    MQTT_MESSAGE_BYTES_RECEIVED,
    "mqtt_message_bytes_received",
    "Total bytes of messages received from clients",
    MessageLabel
);

pub fn record_mqtt_messages_delayed_inc(topic: String) {
    let label = MessageLabel { topic };
    counter_metric_inc!(MQTT_MESSAGES_DELAYED, label);
}

pub fn record_mqtt_messages_received_inc(topic: String) {
    let label = MessageLabel { topic };
    counter_metric_inc!(MQTT_MESSAGES_RECEIVED, label);
}

pub fn record_mqtt_message_bytes_received(topic: String, bytes: u64) {
    let label = MessageLabel { topic };
    counter_metric_inc_by!(MQTT_MESSAGE_BYTES_RECEIVED, label, bytes);
}

pub fn record_mqtt_messages_sent_inc(topic: String) {
    let label = MessageLabel { topic };
    counter_metric_inc!(MQTT_MESSAGES_SENT, label);
}

pub fn record_mqtt_message_bytes_sent(topic: String, bytes: u64) {
    let label = MessageLabel { topic };
    counter_metric_inc_by!(MQTT_MESSAGE_BYTES_SENT, label, bytes);
}
