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
    counter_metric_inc_by, histogram_metric_observe, register_counter_metric,
    register_histogram_metric_ms_with_default_buckets,
};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct ConnectorLabel {
    pub connector_name: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct MessageLabel {}

register_counter_metric!(
    MQTT_CONNECTOR_MESSAGES_SENT_SUCCESS,
    "mqtt_connector_messages_sent_success",
    "Total number of messages successfully sent by connector",
    ConnectorLabel
);

register_counter_metric!(
    MQTT_CONNECTOR_MESSAGES_SENT_FAILURE,
    "mqtt_connector_messages_sent_failure",
    "Total number of messages failed to send by connector",
    ConnectorLabel
);

register_histogram_metric_ms_with_default_buckets!(
    MQTT_CONNECTOR_SEND_DURATION_MS,
    "mqtt_connector_send_duration_ms",
    "Total duration of sending messages by connector in milliseconds",
    ConnectorLabel
);

register_counter_metric!(
    MQTT_CONNECTOR_MESSAGES_SENT_SUCCESS_TOTAL,
    "mqtt_connector_messages_sent_success_total",
    "Total number of messages successfully sent by all connectors",
    MessageLabel
);

register_counter_metric!(
    MQTT_CONNECTOR_MESSAGES_SENT_FAILURE_TOTAL,
    "mqtt_connector_messages_sent_failure_total",
    "Total number of messages failed to send by all connectors",
    MessageLabel
);

register_histogram_metric_ms_with_default_buckets!(
    MQTT_CONNECTOR_SEND_DURATION_MS_TOTAL,
    "mqtt_connector_send_duration_ms_total",
    "Total duration of sending messages by all connectors in milliseconds",
    MessageLabel
);

pub fn record_connector_messages_sent_success(connector_name: String, count: u64) {
    let label = ConnectorLabel { connector_name };
    counter_metric_inc_by!(MQTT_CONNECTOR_MESSAGES_SENT_SUCCESS, label, count);

    let total_label = MessageLabel {};
    counter_metric_inc_by!(
        MQTT_CONNECTOR_MESSAGES_SENT_SUCCESS_TOTAL,
        total_label,
        count
    );
}

pub fn record_connector_messages_sent_failure(connector_name: String, count: u64) {
    let label = ConnectorLabel { connector_name };
    counter_metric_inc_by!(MQTT_CONNECTOR_MESSAGES_SENT_FAILURE, label, count);

    let total_label = MessageLabel {};
    counter_metric_inc_by!(
        MQTT_CONNECTOR_MESSAGES_SENT_FAILURE_TOTAL,
        total_label,
        count
    );
}

pub fn record_connector_send_duration(connector_name: String, duration_ms: f64) {
    let label = ConnectorLabel { connector_name };
    histogram_metric_observe!(MQTT_CONNECTOR_SEND_DURATION_MS, duration_ms, label);

    let total_label = MessageLabel {};
    histogram_metric_observe!(
        MQTT_CONNECTOR_SEND_DURATION_MS_TOTAL,
        duration_ms,
        total_label
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_metrics_with_label() {
        let connector_name = "test_connector".to_string();

        record_connector_messages_sent_success(connector_name.clone(), 10);
        record_connector_messages_sent_success(connector_name.clone(), 5);
        record_connector_messages_sent_failure(connector_name.clone(), 2);
        record_connector_send_duration(connector_name.clone(), 123.45);
        record_connector_send_duration(connector_name.clone(), 67.89);
    }

    #[test]
    fn test_connector_metrics_aggregate() {
        record_connector_messages_sent_success("connector_1".to_string(), 100);
        record_connector_messages_sent_success("connector_2".to_string(), 200);
        record_connector_messages_sent_success("connector_3".to_string(), 300);

        record_connector_messages_sent_failure("connector_1".to_string(), 5);
        record_connector_messages_sent_failure("connector_2".to_string(), 10);

        record_connector_send_duration("connector_1".to_string(), 50.0);
        record_connector_send_duration("connector_2".to_string(), 100.0);
        record_connector_send_duration("connector_3".to_string(), 150.0);
    }
}
