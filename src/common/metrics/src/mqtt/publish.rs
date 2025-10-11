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

use crate::{
    counter_metric_get, counter_metric_inc, counter_metric_inc_by, register_counter_metric,
};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct MessageLabel {}

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

pub fn record_mqtt_messages_delayed_inc() {
    let label = MessageLabel {};
    counter_metric_inc!(MQTT_MESSAGES_DELAYED, label);
}

pub fn record_mqtt_messages_received_inc() {
    let label = MessageLabel {};
    counter_metric_inc!(MQTT_MESSAGES_RECEIVED, label);
}

pub fn record_mqtt_messages_received_get() -> u64 {
    let label = MessageLabel {};
    let mut result = 0u64;
    counter_metric_get!(MQTT_MESSAGES_RECEIVED, label, result);
    result
}

pub fn record_mqtt_message_bytes_received(bytes: u64) {
    let label = MessageLabel {};
    counter_metric_inc_by!(MQTT_MESSAGE_BYTES_RECEIVED, label, bytes);
}

pub fn record_mqtt_messages_sent_inc() {
    let label = MessageLabel {};
    counter_metric_inc!(MQTT_MESSAGES_SENT, label);
}

pub fn record_mqtt_messages_sent_get() -> u64 {
    let label = MessageLabel {};
    let mut result = 0u64;
    counter_metric_get!(MQTT_MESSAGES_SENT, label, result);
    result
}

pub fn record_mqtt_message_bytes_sent(bytes: u64) {
    let label = MessageLabel {};
    counter_metric_inc_by!(MQTT_MESSAGE_BYTES_SENT, label, bytes);
}

register_counter_metric!(
    MQTT_MESSAGES_DROPPED_NO_SUBSCRIBERS,
    "mqtt_messages_dropped_no_subscribers",
    "Number of MQTT messages dropped due to no subscribers",
    MessageLabel
);

pub fn record_messages_dropped_no_subscribers_incr() {
    let label = MessageLabel {};
    counter_metric_inc!(MQTT_MESSAGES_DROPPED_NO_SUBSCRIBERS, label);
}

pub fn record_messages_dropped_no_subscribers_get() -> u64 {
    let label = MessageLabel {};
    let mut result = 0u64;
    counter_metric_get!(MQTT_MESSAGES_DROPPED_NO_SUBSCRIBERS, label, result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mqtt_messages_received_inc_and_get() {
        // Test initial value (should be 0 for a new metric)
        let initial_count = record_mqtt_messages_received_get();

        // Increment the counter
        record_mqtt_messages_received_inc();
        let count_after_one = record_mqtt_messages_received_get();
        assert_eq!(count_after_one, initial_count + 1);

        // Increment multiple times
        record_mqtt_messages_received_inc();
        record_mqtt_messages_received_inc();
        record_mqtt_messages_received_inc();
        let count_after_four = record_mqtt_messages_received_get();
        assert_eq!(count_after_four, initial_count + 4);

        // Verify counter only increases (never decreases)
        let final_count = record_mqtt_messages_received_get();
        assert!(final_count >= initial_count);
    }
}
