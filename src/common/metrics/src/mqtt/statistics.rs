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
    gauge_metric_get, gauge_metric_inc, gauge_metric_inc_by, gauge_metric_set,
    register_gauge_metric,
};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct StatLabel {}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct DelayQueueLabel {
    shard_no: String,
}

register_gauge_metric!(
    MQTT_CONNECTIONS_COUNT,
    "mqtt_connections_count",
    "Current number of MQTT connections",
    StatLabel
);

register_gauge_metric!(
    MQTT_SESSIONS_COUNT,
    "mqtt_sessions_count",
    "Current number of MQTT sessions",
    StatLabel
);

register_gauge_metric!(
    MQTT_TOPICS_COUNT,
    "mqtt_topics_count",
    "Current number of MQTT topics",
    StatLabel
);

register_gauge_metric!(
    MQTT_SUBSCRIBERS_COUNT,
    "mqtt_subscribers_count",
    "Current number of MQTT subscribers",
    StatLabel
);

register_gauge_metric!(
    MQTT_SUBSCRIPTIONS_SHARED_COUNT,
    "mqtt_subscriptions_shared_count",
    "Current number of MQTT shared subscriptions",
    StatLabel
);

register_gauge_metric!(
    MQTT_RETAINED_COUNT,
    "mqtt_retained_count",
    "Current number of MQTT retained messages",
    StatLabel
);

register_gauge_metric!(
    MQTT_DELAY_QUEUE_TOTAL_CAPACITY,
    "mqtt_delay_queue_total_capacity",
    "Total capacity of MQTT delay queue",
    DelayQueueLabel
);

register_gauge_metric!(
    MQTT_DELAY_QUEUE_USED_CAPACITY,
    "mqtt_delay_queue_used_capacity",
    "Used capacity of MQTT delay queue",
    DelayQueueLabel
);

register_gauge_metric!(
    MQTT_DELAY_QUEUE_REMAINING_CAPACITY,
    "mqtt_delay_queue_remaining_capacity",
    "Remaining capacity of MQTT delay queue",
    DelayQueueLabel
);

pub fn record_mqtt_connections_set(count: i64) {
    let label = StatLabel {};
    gauge_metric_set!(MQTT_CONNECTIONS_COUNT, label, count);
}

pub fn record_mqtt_connections_get() -> i64 {
    let label = StatLabel {};
    let mut result = 0i64;
    gauge_metric_get!(MQTT_CONNECTIONS_COUNT, label, result);
    result
}

pub fn record_mqtt_sessions_set(count: i64) {
    let label = StatLabel {};
    gauge_metric_set!(MQTT_SESSIONS_COUNT, label, count);
}

pub fn record_mqtt_sessions_get() -> i64 {
    let label = StatLabel {};
    let mut result = 0i64;
    gauge_metric_get!(MQTT_SESSIONS_COUNT, label, result);
    result
}

pub fn record_mqtt_topics_set(count: i64) {
    let label = StatLabel {};
    gauge_metric_set!(MQTT_TOPICS_COUNT, label, count);
}

pub fn record_mqtt_topics_get() -> i64 {
    let label = StatLabel {};
    let mut result = 0i64;
    gauge_metric_get!(MQTT_TOPICS_COUNT, label, result);
    result
}

pub fn record_mqtt_subscribers_set(count: i64) {
    let label = StatLabel {};
    gauge_metric_set!(MQTT_SUBSCRIBERS_COUNT, label, count);
}

pub fn record_mqtt_subscriptions_shared_set(count: i64) {
    let label = StatLabel {};
    gauge_metric_set!(MQTT_SUBSCRIPTIONS_SHARED_COUNT, label, count);
}

pub fn record_mqtt_subscriptions_shared_get() -> i64 {
    let label = StatLabel {};
    let mut result = 0i64;
    gauge_metric_get!(MQTT_SUBSCRIPTIONS_SHARED_COUNT, label, result);
    result
}

pub fn record_mqtt_retained_inc() {
    let label = StatLabel {};
    gauge_metric_inc!(MQTT_RETAINED_COUNT, label);
}

pub fn record_mqtt_retained_dec() {
    let label = StatLabel {};
    gauge_metric_inc_by!(MQTT_RETAINED_COUNT, label, -1);
}

pub fn record_mqtt_delay_queue_total_capacity_set(shard_no: u64, capacity: i64) {
    let label = DelayQueueLabel {
        shard_no: shard_no.to_string(),
    };
    gauge_metric_set!(MQTT_DELAY_QUEUE_TOTAL_CAPACITY, label, capacity);
}

pub fn record_mqtt_delay_queue_used_capacity_set(shard_no: u64, used: i64) {
    let label = DelayQueueLabel {
        shard_no: shard_no.to_string(),
    };
    gauge_metric_set!(MQTT_DELAY_QUEUE_USED_CAPACITY, label, used);
}

pub fn record_mqtt_delay_queue_remaining_capacity_set(shard_no: u64, remaining: i64) {
    let label = DelayQueueLabel {
        shard_no: shard_no.to_string(),
    };
    gauge_metric_set!(MQTT_DELAY_QUEUE_REMAINING_CAPACITY, label, remaining);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mqtt_connections_set_and_get() {
        // Test initial value (should be 0)
        let initial_count = record_mqtt_connections_get();
        assert_eq!(initial_count, 0);

        // Test setting a value
        record_mqtt_connections_set(100);
        let count = record_mqtt_connections_get();
        assert_eq!(count, 100);

        // Test updating the value
        record_mqtt_connections_set(250);
        let updated_count = record_mqtt_connections_get();
        assert_eq!(updated_count, 250);

        // Test setting to zero
        record_mqtt_connections_set(0);
        let zero_count = record_mqtt_connections_get();
        assert_eq!(zero_count, 0);

        // Test negative value (gauge can store negative values)
        record_mqtt_connections_set(-10);
        let negative_count = record_mqtt_connections_get();
        assert_eq!(negative_count, -10);
    }
}
