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
    counter_metric_get, counter_metric_inc_by, gauge_metric_set, histogram_metric_observe,
    register_counter_metric, register_gauge_metric,
    register_histogram_metric_ms_with_default_buckets,
};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct ConnectorLabel {
    pub connector_type: String,
    pub connector_name: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct MessageLabel {}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct ConnectorStrategyLabel {
    pub connector_type: String,
    pub connector_name: String,
    pub strategy: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct ConnectorResultLabel {
    pub connector_type: String,
    pub connector_name: String,
    pub result: String,
}

macro_rules! get_counter_metric_with_label {
    ($metric:ident, $label_var:ident) => {{
        let mut result = 0u64;
        counter_metric_get!($metric, $label_var, result);
        result
    }};
}

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
    MQTT_CONNECTOR_RETRY_TOTAL,
    "mqtt_connector_retry_total",
    "Total retries executed by connector",
    ConnectorStrategyLabel
);

register_counter_metric!(
    MQTT_CONNECTOR_MESSAGES_DISCARDED_TOTAL,
    "mqtt_connector_messages_discarded_total",
    "Total number of messages discarded by connector terminal strategy",
    ConnectorStrategyLabel
);

register_counter_metric!(
    MQTT_CONNECTOR_DLQ_MESSAGES_TOTAL,
    "mqtt_connector_dlq_messages_total",
    "Total number of messages attempted to write into DLQ by connector",
    ConnectorResultLabel
);

register_counter_metric!(
    MQTT_CONNECTOR_OFFSET_COMMIT_FAILURE_TOTAL,
    "mqtt_connector_offset_commit_failure_total",
    "Total number of offset commit failures by connector",
    ConnectorLabel
);

register_counter_metric!(
    MQTT_CONNECTOR_SOURCE_READ_FAILURE_TOTAL,
    "mqtt_connector_source_read_failure_total",
    "Total number of source read failures by connector",
    ConnectorLabel
);

register_gauge_metric!(
    MQTT_CONNECTOR_UP,
    "mqtt_connector_up",
    "Connector thread health status, 1 means running and 0 means stopped",
    ConnectorLabel
);

register_counter_metric!(
    MQTT_CONNECTOR_MESSAGES_SENT_SUCCESS_TOTAL,
    "mqtt_connector_messages_sent_success_agg",
    "Total number of messages successfully sent by all connectors",
    MessageLabel
);

register_counter_metric!(
    MQTT_CONNECTOR_MESSAGES_SENT_FAILURE_TOTAL,
    "mqtt_connector_messages_sent_failure_agg",
    "Total number of messages failed to send by all connectors",
    MessageLabel
);

register_histogram_metric_ms_with_default_buckets!(
    MQTT_CONNECTOR_SEND_DURATION_MS_TOTAL,
    "mqtt_connector_send_duration_ms_agg",
    "Total duration of sending messages by all connectors in milliseconds",
    MessageLabel
);

pub fn record_connector_messages_sent_success(
    connector_type: String,
    connector_name: String,
    count: u64,
) {
    let label = ConnectorLabel {
        connector_type,
        connector_name,
    };
    counter_metric_inc_by!(MQTT_CONNECTOR_MESSAGES_SENT_SUCCESS, label, count);

    let total_label = MessageLabel {};
    counter_metric_inc_by!(
        MQTT_CONNECTOR_MESSAGES_SENT_SUCCESS_TOTAL,
        total_label,
        count
    );
}

pub fn record_connector_messages_sent_failure(
    connector_type: String,
    connector_name: String,
    count: u64,
) {
    let label = ConnectorLabel {
        connector_type,
        connector_name,
    };
    counter_metric_inc_by!(MQTT_CONNECTOR_MESSAGES_SENT_FAILURE, label, count);

    let total_label = MessageLabel {};
    counter_metric_inc_by!(
        MQTT_CONNECTOR_MESSAGES_SENT_FAILURE_TOTAL,
        total_label,
        count
    );
}

pub fn record_connector_send_duration(
    connector_type: String,
    connector_name: String,
    duration_ms: f64,
) {
    let label = ConnectorLabel {
        connector_type,
        connector_name,
    };
    histogram_metric_observe!(MQTT_CONNECTOR_SEND_DURATION_MS, duration_ms, label);

    let total_label = MessageLabel {};
    histogram_metric_observe!(
        MQTT_CONNECTOR_SEND_DURATION_MS_TOTAL,
        duration_ms,
        total_label
    );
}

pub fn record_connector_retry(
    connector_type: String,
    connector_name: String,
    strategy: &'static str,
) {
    let label = ConnectorStrategyLabel {
        connector_type,
        connector_name,
        strategy: strategy.to_string(),
    };
    counter_metric_inc_by!(MQTT_CONNECTOR_RETRY_TOTAL, label, 1);
}

pub fn record_connector_messages_discarded(
    connector_type: String,
    connector_name: String,
    strategy: &'static str,
    count: u64,
) {
    let label = ConnectorStrategyLabel {
        connector_type,
        connector_name,
        strategy: strategy.to_string(),
    };
    counter_metric_inc_by!(MQTT_CONNECTOR_MESSAGES_DISCARDED_TOTAL, label, count);
}

pub fn record_connector_dlq_messages(
    connector_type: String,
    connector_name: String,
    result: &'static str,
    count: u64,
) {
    let label = ConnectorResultLabel {
        connector_type,
        connector_name,
        result: result.to_string(),
    };
    counter_metric_inc_by!(MQTT_CONNECTOR_DLQ_MESSAGES_TOTAL, label, count);
}

pub fn record_connector_offset_commit_failure(connector_type: String, connector_name: String) {
    let label = ConnectorLabel {
        connector_type,
        connector_name,
    };
    counter_metric_inc_by!(MQTT_CONNECTOR_OFFSET_COMMIT_FAILURE_TOTAL, label, 1);
}

pub fn record_connector_source_read_failure(connector_type: String, connector_name: String) {
    let label = ConnectorLabel {
        connector_type,
        connector_name,
    };
    counter_metric_inc_by!(MQTT_CONNECTOR_SOURCE_READ_FAILURE_TOTAL, label, 1);
}

pub fn set_connector_up(connector_type: String, connector_name: String, up: bool) {
    let label = ConnectorLabel {
        connector_type,
        connector_name,
    };
    gauge_metric_set!(MQTT_CONNECTOR_UP, label, if up { 1 } else { 0 });
}

pub fn get_connector_messages_sent_success(connector_type: &str, connector_name: &str) -> u64 {
    let label = ConnectorLabel {
        connector_type: connector_type.to_string(),
        connector_name: connector_name.to_string(),
    };
    get_counter_metric_with_label!(MQTT_CONNECTOR_MESSAGES_SENT_SUCCESS, label)
}

pub fn get_connector_messages_sent_failure(connector_type: &str, connector_name: &str) -> u64 {
    let label = ConnectorLabel {
        connector_type: connector_type.to_string(),
        connector_name: connector_name.to_string(),
    };
    get_counter_metric_with_label!(MQTT_CONNECTOR_MESSAGES_SENT_FAILURE, label)
}

pub fn get_connector_messages_sent_success_total() -> u64 {
    let label = MessageLabel {};
    get_counter_metric_with_label!(MQTT_CONNECTOR_MESSAGES_SENT_SUCCESS_TOTAL, label)
}

pub fn get_connector_messages_sent_failure_total() -> u64 {
    let label = MessageLabel {};
    get_counter_metric_with_label!(MQTT_CONNECTOR_MESSAGES_SENT_FAILURE_TOTAL, label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_metrics_with_label() {
        let connector_type = "kafka".to_string();
        let connector_name = "test_connector".to_string();

        record_connector_messages_sent_success(connector_type.clone(), connector_name.clone(), 10);
        record_connector_messages_sent_success(connector_type.clone(), connector_name.clone(), 5);
        record_connector_messages_sent_failure(connector_type.clone(), connector_name.clone(), 2);
        record_connector_send_duration(connector_type.clone(), connector_name.clone(), 123.45);
        record_connector_send_duration(connector_type.clone(), connector_name.clone(), 67.89);
        record_connector_retry(
            connector_type.clone(),
            connector_name.clone(),
            "dead_message_queue",
        );
        record_connector_messages_discarded(
            connector_type.clone(),
            connector_name.clone(),
            "discard_after_retry",
            2,
        );
        record_connector_dlq_messages(connector_type.clone(), connector_name.clone(), "success", 2);
        record_connector_offset_commit_failure(connector_type.clone(), connector_name.clone());
        record_connector_source_read_failure(connector_type.clone(), connector_name.clone());
        set_connector_up(connector_type, connector_name, true);
    }

    #[test]
    fn test_connector_metrics_aggregate() {
        record_connector_messages_sent_success("kafka".to_string(), "connector_1".to_string(), 100);
        record_connector_messages_sent_success("kafka".to_string(), "connector_2".to_string(), 200);
        record_connector_messages_sent_success("s3".to_string(), "connector_3".to_string(), 300);

        record_connector_messages_sent_failure("kafka".to_string(), "connector_1".to_string(), 5);
        record_connector_messages_sent_failure("s3".to_string(), "connector_2".to_string(), 10);

        record_connector_send_duration("kafka".to_string(), "connector_1".to_string(), 50.0);
        record_connector_send_duration("kafka".to_string(), "connector_2".to_string(), 100.0);
        record_connector_send_duration("s3".to_string(), "connector_3".to_string(), 150.0);
    }
}
