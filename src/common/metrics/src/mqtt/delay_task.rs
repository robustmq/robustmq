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
    counter_metric_inc, counter_metric_touch, histogram_metric_observe, register_counter_metric,
    register_histogram_metric,
};
use prometheus_client::encoding::EncodeLabelSet;

// ── Labels ──────────────────────────────────────────────────────────────────

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct DelayTaskLabel {}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct DelayTaskTypeLabel {
    pub task_type: String,
}

// ── Metrics ─────────────────────────────────────────────────────────────────

register_counter_metric!(
    DELAY_TASK_CREATED_TOTAL,
    "delay_task_created",
    "Total number of delay tasks created",
    DelayTaskLabel
);

register_counter_metric!(
    DELAY_TASK_EXECUTED_TOTAL,
    "delay_task_executed",
    "Total number of delay tasks executed successfully",
    DelayTaskTypeLabel
);

register_counter_metric!(
    DELAY_TASK_EXECUTE_FAILED_TOTAL,
    "delay_task_execute_failed",
    "Total number of delay task execution failures",
    DelayTaskTypeLabel
);

register_histogram_metric!(
    DELAY_TASK_SCHEDULE_LATENCY_S,
    "delay_task_schedule_latency_seconds",
    "Difference between actual execution time and target delay time in seconds",
    DelayTaskTypeLabel,
    [0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0]
);

// ── Public API ──────────────────────────────────────────────────────────────

pub fn record_delay_task_created() {
    let l = DelayTaskLabel {};
    counter_metric_inc!(DELAY_TASK_CREATED_TOTAL, l);
}

pub fn record_delay_task_executed(task_type: &str) {
    let l = DelayTaskTypeLabel {
        task_type: task_type.to_string(),
    };
    counter_metric_inc!(DELAY_TASK_EXECUTED_TOTAL, l);
}

pub fn record_delay_task_execute_failed(task_type: &str) {
    let l = DelayTaskTypeLabel {
        task_type: task_type.to_string(),
    };
    counter_metric_inc!(DELAY_TASK_EXECUTE_FAILED_TOTAL, l);
}

pub fn record_delay_task_schedule_latency(task_type: &str, latency_s: f64) {
    let l = DelayTaskTypeLabel {
        task_type: task_type.to_string(),
    };
    histogram_metric_observe!(DELAY_TASK_SCHEDULE_LATENCY_S, latency_s, l);
}

pub fn init() {
    counter_metric_touch!(DELAY_TASK_CREATED_TOTAL, DelayTaskLabel {});
}
