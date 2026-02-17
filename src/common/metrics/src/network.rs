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
    gauge_metric_inc_by, gauge_metric_set, histogram_metric_observe, register_gauge_metric,
    register_histogram_metric_ms_with_default_buckets,
};
use common_base::tools::now_millis;
use metadata_struct::connection::NetworkConnectionType;
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct LabelType {
    label: String,
}

register_gauge_metric!(
    BROKER_NETWORK_REQUEST_QUEUE_BLOCK_NUM,
    "network_request_queue_block_num",
    "broker request network queue blocked message num",
    LabelType
);

register_gauge_metric!(
    BROKER_NETWORK_REQUEST_QUEUE_REMAINING_NUM,
    "network_request_queue_remaining_num",
    "broker request network queue remaining capacity",
    LabelType
);

register_gauge_metric!(
    BROKER_NETWORK_REQUEST_QUEUE_USE_NUM,
    "network_request_queue_use_num",
    "broker request network queue used capacity",
    LabelType
);

register_gauge_metric!(
    BROKER_NETWORK_RESPONSE_QUEUE_BLOCK_NUM,
    "network_response_queue_block_num",
    "broker response network queue blocked message num",
    LabelType
);

register_gauge_metric!(
    BROKER_NETWORK_RESPONSE_QUEUE_REMAINING_NUM,
    "network_response_queue_remaining_num",
    "broker response network queue remaining capacity",
    LabelType
);

register_gauge_metric!(
    BROKER_NETWORK_RESPONSE_QUEUE_USE_NUM,
    "network_response_queue_use_num",
    "broker response network queue used capacity",
    LabelType
);

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct NetworkLabel {
    network: String,
}

register_histogram_metric_ms_with_default_buckets!(
    REQUEST_TOTAL_MS,
    "request_total_ms",
    "The total duration of request packets processed in the broker",
    NetworkLabel
);

register_histogram_metric_ms_with_default_buckets!(
    REQUEST_NOT_RESPONSE_TOTAL_MS,
    "request_not_response_total_ms",
    "The not response total duration of request packets processed in the broker",
    NetworkLabel
);

register_histogram_metric_ms_with_default_buckets!(
    REQUEST_QUEUE_MS,
    "request_queue_ms",
    "The total duration of request packets in the broker queue",
    NetworkLabel
);

register_histogram_metric_ms_with_default_buckets!(
    REQUEST_HANDLER_MS,
    "request_handler_ms",
    "The total duration of request packets handle in the broker",
    NetworkLabel
);

register_histogram_metric_ms_with_default_buckets!(
    REQUEST_RESPONSE_MS,
    "request_response_ms",
    "The total duration of request packets response in the broker",
    NetworkLabel
);

register_histogram_metric_ms_with_default_buckets!(
    REQUEST_RESPONSE_QUEUE_MS,
    "request_response_queue_ms",
    "The total duration of request packets response queue in the broker",
    NetworkLabel
);

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct BrokerThreadLabel {
    network: String,
    thread_type: String,
}

register_gauge_metric!(
    BROKER_ACTIVE_THREAD_NUM,
    "broker_active_thread_num",
    "The number of execution active threads started by the broker",
    BrokerThreadLabel
);

pub fn metrics_request_total_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    histogram_metric_observe!(REQUEST_TOTAL_MS, ms, label);
}

pub fn metrics_request_not_response_total_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    histogram_metric_observe!(REQUEST_NOT_RESPONSE_TOTAL_MS, ms, label);
}

pub fn metrics_request_queue_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    histogram_metric_observe!(REQUEST_QUEUE_MS, ms, label);
}

pub fn metrics_request_handler_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    histogram_metric_observe!(REQUEST_HANDLER_MS, ms, label);
}

pub fn metrics_request_response_queue_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    histogram_metric_observe!(REQUEST_RESPONSE_QUEUE_MS, ms, label);
}

pub fn metrics_request_response_ms(network_connection: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network_connection.to_string(),
    };
    histogram_metric_observe!(REQUEST_RESPONSE_MS, ms, label);
}

pub fn metrics_request_queue_size(
    label: &str,
    block_num: usize,
    use_num: usize,
    remaining_num: usize,
) {
    let label_type = LabelType {
        label: label.to_string(),
    };

    let label_block = label_type.clone();
    gauge_metric_set!(
        BROKER_NETWORK_REQUEST_QUEUE_BLOCK_NUM,
        label_block,
        block_num as i64
    );

    let label_use = label_type.clone();
    gauge_metric_set!(
        BROKER_NETWORK_REQUEST_QUEUE_USE_NUM,
        label_use,
        use_num as i64
    );

    gauge_metric_set!(
        BROKER_NETWORK_REQUEST_QUEUE_REMAINING_NUM,
        label_type,
        remaining_num as i64
    );
}

pub fn metrics_response_queue_size(
    label: &str,
    block_num: usize,
    use_num: usize,
    remaining_num: usize,
) {
    let label_type = LabelType {
        label: label.to_string(),
    };

    let label_block = label_type.clone();
    gauge_metric_set!(
        BROKER_NETWORK_RESPONSE_QUEUE_BLOCK_NUM,
        label_block,
        block_num as i64
    );

    let label_use = label_type.clone();
    gauge_metric_set!(
        BROKER_NETWORK_RESPONSE_QUEUE_USE_NUM,
        label_use,
        use_num as i64
    );

    gauge_metric_set!(
        BROKER_NETWORK_RESPONSE_QUEUE_REMAINING_NUM,
        label_type,
        remaining_num as i64
    );
}

pub fn record_ws_request_duration(receive_ms: u128, response_ms: u128) {
    let now = now_millis();
    let ws_type = NetworkConnectionType::WebSocket;

    metrics_request_total_ms(&ws_type, (now - receive_ms) as f64);
    metrics_request_handler_ms(&ws_type, (response_ms - receive_ms) as f64);
    metrics_request_response_ms(&ws_type, (now - response_ms) as f64);
}

pub fn record_broker_thread_num(
    network_connection: &NetworkConnectionType,
    (accept, handler): (usize, usize),
) {
    let accept_label = BrokerThreadLabel {
        network: network_connection.to_string(),
        thread_type: "accept".to_string(),
    };
    gauge_metric_inc_by!(BROKER_ACTIVE_THREAD_NUM, accept_label, accept as i64);

    let handler_label = BrokerThreadLabel {
        network: network_connection.to_string(),
        thread_type: "handler".to_string(),
    };
    gauge_metric_inc_by!(BROKER_ACTIVE_THREAD_NUM, handler_label, handler as i64);
}
