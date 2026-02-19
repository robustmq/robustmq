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
use metadata_struct::connection::NetworkConnectionType;
use prometheus_client::encoding::EncodeLabelSet;

// ── Labels ──────────────────────────────────────────────────────────────────

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct NetworkLabel {
    network: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct QueueLabel {
    label: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct BrokerThreadLabel {
    network: String,
    thread_type: String,
}

// ── Handler latency histograms ──────────────────────────────────────────────

register_histogram_metric_ms_with_default_buckets!(
    HANDLER_QUEUE_WAIT_MS,
    "handler_queue_wait_ms",
    "Time a request spent waiting in the handler queue before being picked up (ms)",
    NetworkLabel
);

register_histogram_metric_ms_with_default_buckets!(
    HANDLER_APPLY_MS,
    "handler_apply_ms",
    "Time spent in command.apply() processing the request (ms)",
    NetworkLabel
);

register_histogram_metric_ms_with_default_buckets!(
    HANDLER_WRITE_MS,
    "handler_write_ms",
    "Time spent writing the response back to the client (ms)",
    NetworkLabel
);

register_histogram_metric_ms_with_default_buckets!(
    HANDLER_TOTAL_MS,
    "handler_total_ms",
    "Total time from request received to response written (ms)",
    NetworkLabel
);

// ── Handler queue gauges ────────────────────────────────────────────────────

register_gauge_metric!(
    HANDLER_QUEUE_SIZE,
    "handler_queue_size",
    "Current number of pending requests in the handler queue",
    QueueLabel
);

register_gauge_metric!(
    HANDLER_QUEUE_REMAINING,
    "handler_queue_remaining",
    "Remaining capacity in the handler queue",
    QueueLabel
);

// ── Handler counters (gauge used as counter) ────────────────────────────────

register_gauge_metric!(
    HANDLER_REQUESTS_TOTAL,
    "handler_requests_total",
    "Total number of requests processed by handlers",
    NetworkLabel
);

register_gauge_metric!(
    HANDLER_SLOW_REQUESTS_TOTAL,
    "handler_slow_requests_total",
    "Total number of slow requests (exceeding threshold)",
    NetworkLabel
);

register_gauge_metric!(
    HANDLER_TIMEOUT_TOTAL,
    "handler_timeout_total",
    "Total number of handler apply timeouts (requests exceeding the hard deadline)",
    NetworkLabel
);

// ── Per-handler-instance metrics ────────────────────────────────────────────

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct HandlerIndexLabel {
    handler: String,
}

register_gauge_metric!(
    HANDLER_INSTANCE_REQUESTS_TOTAL,
    "handler_instance_requests_total",
    "Total requests processed by each individual handler thread",
    HandlerIndexLabel
);

register_histogram_metric_ms_with_default_buckets!(
    HANDLER_INSTANCE_APPLY_MS,
    "handler_instance_apply_ms",
    "command.apply() latency per individual handler thread (ms)",
    HandlerIndexLabel
);

// ── Thread gauge ────────────────────────────────────────────────────────────

register_gauge_metric!(
    BROKER_ACTIVE_THREAD_NUM,
    "broker_active_thread_num",
    "The number of execution active threads started by the broker",
    BrokerThreadLabel
);

// ── Public recording functions ──────────────────────────────────────────────

pub fn metrics_handler_queue_wait_ms(network: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network.to_string(),
    };
    histogram_metric_observe!(HANDLER_QUEUE_WAIT_MS, ms, label);
}

pub fn metrics_handler_apply_ms(network: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network.to_string(),
    };
    histogram_metric_observe!(HANDLER_APPLY_MS, ms, label);
}

pub fn metrics_handler_write_ms(network: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network.to_string(),
    };
    histogram_metric_observe!(HANDLER_WRITE_MS, ms, label);
}

pub fn metrics_handler_total_ms(network: &NetworkConnectionType, ms: f64) {
    let label = NetworkLabel {
        network: network.to_string(),
    };
    histogram_metric_observe!(HANDLER_TOTAL_MS, ms, label);
}

pub fn metrics_handler_queue_state(current_len: usize, capacity: usize) {
    let label_size = QueueLabel {
        label: "handler".to_string(),
    };
    let label_remaining = QueueLabel {
        label: "handler".to_string(),
    };
    gauge_metric_set!(HANDLER_QUEUE_SIZE, label_size, current_len as i64);
    gauge_metric_set!(
        HANDLER_QUEUE_REMAINING,
        label_remaining,
        capacity.saturating_sub(current_len) as i64
    );
}

pub fn metrics_handler_request_count(network: &NetworkConnectionType) {
    let label = NetworkLabel {
        network: network.to_string(),
    };
    gauge_metric_inc_by!(HANDLER_REQUESTS_TOTAL, label, 1);
}

pub fn metrics_handler_slow_request_count(network: &NetworkConnectionType) {
    let label = NetworkLabel {
        network: network.to_string(),
    };
    gauge_metric_inc_by!(HANDLER_SLOW_REQUESTS_TOTAL, label, 1);
}

pub fn metrics_handler_timeout_count(network: &NetworkConnectionType) {
    let label = NetworkLabel {
        network: network.to_string(),
    };
    gauge_metric_inc_by!(HANDLER_TIMEOUT_TOTAL, label, 1);
}

pub fn metrics_handler_instance_request_count(handler_index: usize) {
    let label = HandlerIndexLabel {
        handler: handler_index.to_string(),
    };
    gauge_metric_inc_by!(HANDLER_INSTANCE_REQUESTS_TOTAL, label, 1);
}

pub fn metrics_handler_instance_apply_ms(handler_index: usize, ms: f64) {
    let label = HandlerIndexLabel {
        handler: handler_index.to_string(),
    };
    histogram_metric_observe!(HANDLER_INSTANCE_APPLY_MS, ms, label);
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
