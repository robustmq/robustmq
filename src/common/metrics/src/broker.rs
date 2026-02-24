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

use crate::{gauge_metric_get, gauge_metric_set, register_gauge_metric};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
struct SystemLabel {}

/// Label used to distinguish metrics per Tokio runtime (server / meta / broker).
#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct RuntimeLabel {
    pub runtime: String,
}

register_gauge_metric!(
    TOKIO_RUNTIME_BUSY_RATIO,
    "tokio_runtime_busy_ratio",
    "Worker-thread busy ratio for the named Tokio runtime (0-100)",
    RuntimeLabel
);

// ── Tokio runtime: global queue depth ─────────────────────────────────────
register_gauge_metric!(
    TOKIO_RUNTIME_QUEUE_DEPTH,
    "tokio_runtime_queue_depth",
    "Number of tasks waiting in the global queue of the named Tokio runtime",
    RuntimeLabel
);

// ── Tokio runtime: alive tasks ────────────────────────────────────────────
register_gauge_metric!(
    TOKIO_RUNTIME_ALIVE_TASKS,
    "tokio_runtime_alive_tasks",
    "Number of alive (spawned but not yet completed) tasks in the named Tokio runtime",
    RuntimeLabel
);

register_gauge_metric!(
    SYSTEM_PROCESS_CPU_USAGE,
    "system_process_cpu_usage",
    "CPU usage percentage of the current process (0-100, normalized by core count)",
    SystemLabel
);

register_gauge_metric!(
    SYSTEM_PROCESS_MEMORY_USAGE,
    "system_process_memory_usage",
    "Memory usage percentage of the current process relative to total system memory (0-100)",
    SystemLabel
);

register_gauge_metric!(
    SYSTEM_CPU_USAGE,
    "system_cpu_usage",
    "Overall system CPU usage percentage (0-100)",
    SystemLabel
);

register_gauge_metric!(
    SYSTEM_MEMORY_USAGE,
    "system_memory_usage",
    "Overall system memory usage percentage (0-100)",
    SystemLabel
);

pub fn record_system_process_cpu_set(value: i64) {
    let label = SystemLabel {};
    gauge_metric_set!(SYSTEM_PROCESS_CPU_USAGE, label, value);
}

pub fn record_system_process_cpu_get() -> i64 {
    let label = SystemLabel {};
    let mut result = 0i64;
    gauge_metric_get!(SYSTEM_PROCESS_CPU_USAGE, label, result);
    result
}

pub fn record_system_process_memory_set(value: i64) {
    let label = SystemLabel {};
    gauge_metric_set!(SYSTEM_PROCESS_MEMORY_USAGE, label, value);
}

pub fn record_system_process_memory_get() -> i64 {
    let label = SystemLabel {};
    let mut result = 0i64;
    gauge_metric_get!(SYSTEM_PROCESS_MEMORY_USAGE, label, result);
    result
}

pub fn record_system_cpu_set(value: i64) {
    let label = SystemLabel {};
    gauge_metric_set!(SYSTEM_CPU_USAGE, label, value);
}

pub fn record_system_cpu_get() -> i64 {
    let label = SystemLabel {};
    let mut result = 0i64;
    gauge_metric_get!(SYSTEM_CPU_USAGE, label, result);
    result
}

pub fn record_system_memory_set(value: i64) {
    let label = SystemLabel {};
    gauge_metric_set!(SYSTEM_MEMORY_USAGE, label, value);
}

pub fn record_system_memory_get() -> i64 {
    let label = SystemLabel {};
    let mut result = 0i64;
    gauge_metric_get!(SYSTEM_MEMORY_USAGE, label, result);
    result
}

pub fn record_runtime_busy_ratio_set(runtime: &str, value: i64) {
    let label = RuntimeLabel {
        runtime: runtime.to_string(),
    };
    gauge_metric_set!(TOKIO_RUNTIME_BUSY_RATIO, label, value);
}

pub fn record_runtime_queue_depth_set(runtime: &str, value: i64) {
    let label = RuntimeLabel {
        runtime: runtime.to_string(),
    };
    gauge_metric_set!(TOKIO_RUNTIME_QUEUE_DEPTH, label, value);
}

pub fn record_runtime_alive_tasks_set(runtime: &str, value: i64) {
    let label = RuntimeLabel {
        runtime: runtime.to_string(),
    };
    gauge_metric_set!(TOKIO_RUNTIME_ALIVE_TASKS, label, value);
}

/// Pre-register all gauge metrics in this module to 0 so they appear in
/// the Prometheus output immediately on startup.
pub fn init() {
    let label = SystemLabel {};
    gauge_metric_set!(SYSTEM_PROCESS_CPU_USAGE, label, 0);
    let label = SystemLabel {};
    gauge_metric_set!(SYSTEM_PROCESS_MEMORY_USAGE, label, 0);
    let label = SystemLabel {};
    gauge_metric_set!(SYSTEM_CPU_USAGE, label, 0);
    let label = SystemLabel {};
    gauge_metric_set!(SYSTEM_MEMORY_USAGE, label, 0);

    for rt in &["server", "meta", "broker"] {
        let label = RuntimeLabel {
            runtime: rt.to_string(),
        };
        gauge_metric_set!(TOKIO_RUNTIME_BUSY_RATIO, label, 0);
        let label = RuntimeLabel {
            runtime: rt.to_string(),
        };
        gauge_metric_set!(TOKIO_RUNTIME_QUEUE_DEPTH, label, 0);
        let label = RuntimeLabel {
            runtime: rt.to_string(),
        };
        gauge_metric_set!(TOKIO_RUNTIME_ALIVE_TASKS, label, 0);
    }
}
