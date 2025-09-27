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

use prometheus_client::encoding::EncodeLabelSet;

use crate::{counter_metric_get, counter_metric_inc, register_counter_metric};

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct ClientConnectionLabels {
    client_id: String,
}

register_counter_metric!(
    CLIENT_CONNECTION_COUNTER,
    "client_connections",
    "The number of client connections, regardless of success or failure.",
    ClientConnectionLabels
);

pub fn incr_client_connection_counter(client_id: String) {
    let labels = ClientConnectionLabels { client_id };
    counter_metric_inc!(CLIENT_CONNECTION_COUNTER, labels)
}

pub fn get_client_connection_counter(client_id: String) -> u64 {
    let labels = ClientConnectionLabels { client_id };
    let mut res = 0;
    counter_metric_get!(CLIENT_CONNECTION_COUNTER, labels, res);
    res
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct EventLabel {}

register_counter_metric!(
    MQTT_CONNECTION_SUCCESS,
    "mqtt_connection_success",
    "Number of successful MQTT connections",
    EventLabel
);

register_counter_metric!(
    MQTT_CONNECTION_FAILED,
    "mqtt_connection_failed",
    "Number of failed MQTT connections",
    EventLabel
);

register_counter_metric!(
    MQTT_DISCONNECT_SUCCESS,
    "mqtt_disconnect_success",
    "Number of successful MQTT disconnections",
    EventLabel
);

register_counter_metric!(
    MQTT_CONNECTION_EXPIRED,
    "mqtt_connection_expired",
    "Number of expired MQTT connections",
    EventLabel
);

register_counter_metric!(
    MQTT_SUBSCRIBE_SUCCESS,
    "mqtt_subscribe_success",
    "Number of successful MQTT subscriptions",
    EventLabel
);

register_counter_metric!(
    MQTT_UNSUBSCRIBE_SUCCESS,
    "mqtt_unsubscribe_success",
    "Number of successful MQTT unsubscriptions",
    EventLabel
);

register_counter_metric!(
    MQTT_SUBSCRIBE_FAILED,
    "mqtt_subscribe_failed",
    "Number of failed MQTT subscriptions",
    EventLabel
);

pub fn record_mqtt_connection_success() {
    let label = EventLabel {};
    counter_metric_inc!(MQTT_CONNECTION_SUCCESS, label);
}

pub fn record_mqtt_connection_failed() {
    let label = EventLabel {};
    counter_metric_inc!(MQTT_CONNECTION_FAILED, label);
}

pub fn record_mqtt_disconnect_success() {
    let label = EventLabel {};
    counter_metric_inc!(MQTT_DISCONNECT_SUCCESS, label);
}

pub fn record_mqtt_connection_expired() {
    let label = EventLabel {};
    counter_metric_inc!(MQTT_CONNECTION_EXPIRED, label);
}

pub fn record_mqtt_subscribe_success() {
    let label = EventLabel {};
    counter_metric_inc!(MQTT_SUBSCRIBE_SUCCESS, label);
}

pub fn record_mqtt_unsubscribe_success() {
    let label = EventLabel {};
    counter_metric_inc!(MQTT_UNSUBSCRIBE_SUCCESS, label);
}

pub fn record_mqtt_subscribe_failed() {
    let label = EventLabel {};
    counter_metric_inc!(MQTT_SUBSCRIBE_FAILED, label);
}

#[cfg(test)]
mod tests {
    use crate::mqtt::event;

    #[test]
    fn test_incr_client_connection_counter() {
        event::incr_client_connection_counter("test_client_1".to_string());

        assert_eq!(
            event::get_client_connection_counter("test_client_1".to_string()),
            1
        );

        event::incr_client_connection_counter("test_client_1".to_string());

        assert_eq!(
            event::get_client_connection_counter("test_client_1".to_string()),
            2
        );

        event::incr_client_connection_counter("test_client_2".to_string());

        assert_eq!(
            event::get_client_connection_counter("test_client_2".to_string()),
            1
        );
    }
}
