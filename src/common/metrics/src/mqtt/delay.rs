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
    counter_metric_inc, histogram_metric_observe, register_counter_metric,
    register_histogram_metric_ms_with_default_buckets,
};
use prometheus_client::encoding::EncodeLabelSet;

// ── Labels ──────────────────────────────────────────────────────────────────

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct DelayLabel {}

// ── Metrics ─────────────────────────────────────────────────────────────────

register_counter_metric!(
    DELAY_MSG_ENQUEUE_TOTAL,
    "delay_msg_enqueue_total",
    "Total number of messages enqueued into the delay queue",
    DelayLabel
);

register_histogram_metric_ms_with_default_buckets!(
    DELAY_MSG_ENQUEUE_DURATION_MS,
    "delay_msg_enqueue_duration_ms",
    "Duration of enqueuing a message into the delay queue in milliseconds",
    DelayLabel
);

register_counter_metric!(
    DELAY_MSG_DELIVER_TOTAL,
    "delay_msg_deliver_total",
    "Total number of delay messages delivered to target topic",
    DelayLabel
);

register_counter_metric!(
    DELAY_MSG_DELIVER_FAIL_TOTAL,
    "delay_msg_deliver_fail_total",
    "Total number of delay message delivery failures",
    DelayLabel
);

register_histogram_metric_ms_with_default_buckets!(
    DELAY_MSG_DELIVER_DURATION_MS,
    "delay_msg_deliver_duration_ms",
    "Duration of delivering a delay message to target topic in milliseconds",
    DelayLabel
);

register_counter_metric!(
    DELAY_MSG_RECOVER_TOTAL,
    "delay_msg_recover_total",
    "Total number of delay messages recovered from persistent storage on startup",
    DelayLabel
);

register_counter_metric!(
    DELAY_MSG_RECOVER_EXPIRED_TOTAL,
    "delay_msg_recover_expired_total",
    "Total number of already-expired delay messages found during recovery",
    DelayLabel
);

// ── Public API ──────────────────────────────────────────────────────────────

fn label() -> DelayLabel {
    DelayLabel {}
}

pub fn record_delay_msg_enqueue() {
    let l = label();
    counter_metric_inc!(DELAY_MSG_ENQUEUE_TOTAL, l);
}

pub fn record_delay_msg_enqueue_duration(duration_ms: f64) {
    let l = label();
    histogram_metric_observe!(DELAY_MSG_ENQUEUE_DURATION_MS, duration_ms, l);
}

pub fn record_delay_msg_deliver() {
    let l = label();
    counter_metric_inc!(DELAY_MSG_DELIVER_TOTAL, l);
}

pub fn record_delay_msg_deliver_fail() {
    let l = label();
    counter_metric_inc!(DELAY_MSG_DELIVER_FAIL_TOTAL, l);
}

pub fn record_delay_msg_deliver_duration(duration_ms: f64) {
    let l = label();
    histogram_metric_observe!(DELAY_MSG_DELIVER_DURATION_MS, duration_ms, l);
}

pub fn record_delay_msg_recover() {
    let l = label();
    counter_metric_inc!(DELAY_MSG_RECOVER_TOTAL, l);
}

pub fn record_delay_msg_recover_expired() {
    let l = label();
    counter_metric_inc!(DELAY_MSG_RECOVER_EXPIRED_TOTAL, l);
}
