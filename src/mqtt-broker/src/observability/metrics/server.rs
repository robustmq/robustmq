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

use crate::server::connection::NetworkConnectionType;
use common_base::tools::now_mills;
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct LabelType {
    label: String,
    r#type: String,
}

common_base::register_gauge_metric!(
    BROKER_NETWORK_QUEUE_NUM,
    "network_queue_num",
    "broker network queue num",
    LabelType
);

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct NetworkLabel {
    network: String,
}

common_base::register_histogram_metric!(
    REQUEST_TOTAL_MS,
    "request.total.ms",
    "The total duration of request packets processed in the broker",
    NetworkLabel,
    5.0,
    2.0,
    12
);

common_base::register_histogram_metric!(
    REQUEST_QUEUE_MS,
    "request.queue.ms",
    "The total duration of request packets in the broker queue",
    NetworkLabel,
    5.0,
    2.0,
    12
);

common_base::register_histogram_metric!(
    REQUEST_HANDLER_MS,
    "request.handler.ms",
    "The total duration of request packets handle in the broker",
    NetworkLabel,
    5.0,
    2.0,
    12
);

common_base::register_histogram_metric!(
    REQUEST_RESPONSE_MS,
    "request.response.ms",
    "The total duration of request packets response in the broker",
    NetworkLabel,
    5.0,
    2.0,
    12
);

common_base::register_histogram_metric!(
    REQUEST_RESPONSE_QUEUE_MS,
    "request.response.queue_ms",
    "The total duration of request packets response queue in the broker",
    NetworkLabel,
    5.0,
    2.0,
    12
);

pub fn metrics_request_total_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    common_base::histogram_metric_observe!(REQUEST_TOTAL_MS, ms, label);
}

pub fn metrics_request_queue_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    common_base::histogram_metric_observe!(REQUEST_QUEUE_MS, ms, label);
}

pub fn metrics_request_handler_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    common_base::histogram_metric_observe!(REQUEST_HANDLER_MS, ms, label);
}

pub fn metrics_request_response_queue_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    common_base::histogram_metric_observe!(REQUEST_RESPONSE_QUEUE_MS, ms, label);
}

pub fn metrics_request_response_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    common_base::histogram_metric_observe!(REQUEST_RESPONSE_MS, ms, label);
}

pub fn metrics_request_queue_size(label: &str, len: usize) {
    let label_type = LabelType {
        label: label.to_string(),
        r#type: "request".to_string(),
    };
    common_base::gauge_metric_inc_by!(BROKER_NETWORK_QUEUE_NUM, label_type, len as i64);
}

pub fn metrics_response_queue_size(label: &str, len: usize) {
    let label_type: LabelType = LabelType {
        label: label.to_string(),
        r#type: "response".to_string(),
    };
    common_base::gauge_metric_inc_by!(BROKER_NETWORK_QUEUE_NUM, label_type, len as i64);
}

pub fn record_response_and_total_ms(
    connection_type: &NetworkConnectionType,
    receive_ms: u128,
    response_ms: u128,
) {
    let now = now_mills();
    metrics_request_total_ms(connection_type, (now - receive_ms) as f64);
    metrics_request_response_ms(connection_type, (now - response_ms) as f64);
}

pub fn record_ws_request_duration(receive_ms: u128, response_ms: u128) {
    let now = now_mills();
    let ws_type = NetworkConnectionType::WebSocket;

    metrics_request_total_ms(&ws_type, (now - receive_ms) as f64);
    metrics_request_handler_ms(&ws_type, (response_ms - receive_ms) as f64);
    metrics_request_response_ms(&ws_type, (now - response_ms) as f64);
}
