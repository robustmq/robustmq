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
    counter_metric_inc, counter_metric_touch, gauge_metric_set, histogram_metric_observe,
    histogram_metric_touch, register_counter_metric, register_gauge_metric,
    register_histogram_metric_ms_with_default_buckets,
};
use prometheus_client::encoding::EncodeLabelSet;

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct RaftLabel {
    pub machine: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct RaftRpcLabel {
    pub machine: String,
    pub rpc_type: String,
}

register_counter_metric!(
    RAFT_WRITE_REQUESTS_TOTAL,
    "raft_write_requests",
    "Total number of write requests",
    RaftLabel
);

register_counter_metric!(
    RAFT_WRITE_SUCCESS_TOTAL,
    "raft_write_success",
    "Total number of successful writes",
    RaftLabel
);

register_counter_metric!(
    RAFT_WRITE_FAILURES_TOTAL,
    "raft_write_failures",
    "Total number of failed writes",
    RaftLabel
);

register_histogram_metric_ms_with_default_buckets!(
    RAFT_WRITE_DURATION,
    "raft_write_duration_ms",
    "Duration of write operations in milliseconds",
    RaftLabel
);

register_gauge_metric!(
    RAFT_APPLY_LAG,
    "raft_apply_lag",
    "Gap between last_log_index and last_applied index; non-zero means state machine is behind",
    RaftLabel
);

register_gauge_metric!(
    RAFT_LAST_LOG_INDEX,
    "raft_last_log_index",
    "Latest log index appended to the Raft log",
    RaftLabel
);

register_gauge_metric!(
    RAFT_LAST_APPLIED,
    "raft_last_applied",
    "Latest log index applied to the state machine",
    RaftLabel
);

register_histogram_metric_ms_with_default_buckets!(
    RAFT_APPLY_BATCH_DURATION,
    "raft_apply_batch_duration_ms",
    "Total duration of a single apply() batch call in milliseconds",
    RaftLabel
);

register_histogram_metric_ms_with_default_buckets!(
    RAFT_LOG_APPEND_BATCH_DURATION,
    "raft_log_append_batch_duration_ms",
    "Total duration of a single log append() call in milliseconds",
    RaftLabel
);

register_counter_metric!(
    RAFT_RPC_REQUESTS_TOTAL,
    "raft_rpc_requests",
    "Total number of RPC requests",
    RaftRpcLabel
);

register_counter_metric!(
    RAFT_RPC_SUCCESS_TOTAL,
    "raft_rpc_success",
    "Total number of successful RPC requests",
    RaftRpcLabel
);

register_counter_metric!(
    RAFT_RPC_FAILURES_TOTAL,
    "raft_rpc_failures",
    "Total number of failed RPC requests",
    RaftRpcLabel
);

register_histogram_metric_ms_with_default_buckets!(
    RAFT_RPC_DURATION,
    "raft_rpc_duration_ms",
    "Duration of RPC operations in milliseconds",
    RaftRpcLabel
);

pub fn record_write_request(machine: &str) {
    let label = RaftLabel {
        machine: machine.to_string(),
    };
    counter_metric_inc!(RAFT_WRITE_REQUESTS_TOTAL, label);
}

pub fn record_write_success(machine: &str) {
    let label = RaftLabel {
        machine: machine.to_string(),
    };
    counter_metric_inc!(RAFT_WRITE_SUCCESS_TOTAL, label);
}

pub fn record_write_failure(machine: &str) {
    let label = RaftLabel {
        machine: machine.to_string(),
    };
    counter_metric_inc!(RAFT_WRITE_FAILURES_TOTAL, label);
}

pub fn record_write_duration(machine: &str, duration_ms: f64) {
    let label = RaftLabel {
        machine: machine.to_string(),
    };
    histogram_metric_observe!(RAFT_WRITE_DURATION, duration_ms, label);
}

pub fn record_rpc_request(machine: &str, rpc_type: &str) {
    let label = RaftRpcLabel {
        machine: machine.to_string(),
        rpc_type: rpc_type.to_string(),
    };
    counter_metric_inc!(RAFT_RPC_REQUESTS_TOTAL, label);
}

pub fn record_rpc_success(machine: &str, rpc_type: &str) {
    let label = RaftRpcLabel {
        machine: machine.to_string(),
        rpc_type: rpc_type.to_string(),
    };
    counter_metric_inc!(RAFT_RPC_SUCCESS_TOTAL, label);
}

pub fn record_rpc_failure(machine: &str, rpc_type: &str) {
    let label = RaftRpcLabel {
        machine: machine.to_string(),
        rpc_type: rpc_type.to_string(),
    };
    counter_metric_inc!(RAFT_RPC_FAILURES_TOTAL, label);
}

pub fn record_rpc_duration(machine: &str, rpc_type: &str, duration_ms: f64) {
    let label = RaftRpcLabel {
        machine: machine.to_string(),
        rpc_type: rpc_type.to_string(),
    };
    histogram_metric_observe!(RAFT_RPC_DURATION, duration_ms, label);
}

/// Pre-register Raft metrics for all known state machines (called at broker init).
/// Only registers the static `metadata_0` shard; dynamic shard counts are registered
/// via `init_raft_shards` once the config is available.
pub fn init() {
    init_raft_shards(1, 1);
}

/// Pre-register Raft metrics for all shards based on configured shard counts.
/// Call this from MultiRaftManager::new() after reading config.
/// Shard names: metadata_0, offset_0..offset_{n-1}, data_0..data_{n-1}
pub fn init_raft_shards(offset_group_num: u32, data_group_num: u32) {
    let mut shards: Vec<String> = Vec::new();
    shards.push("metadata_0".to_string());
    for i in 0..offset_group_num {
        shards.push(format!("offset_{}", i));
    }
    for i in 0..data_group_num {
        shards.push(format!("data_{}", i));
    }

    for shard in &shards {
        let label = RaftLabel { machine: shard.clone() };
        gauge_metric_set!(RAFT_APPLY_LAG, label, 0);
        let label = RaftLabel { machine: shard.clone() };
        gauge_metric_set!(RAFT_LAST_LOG_INDEX, label, 0);
        let label = RaftLabel { machine: shard.clone() };
        gauge_metric_set!(RAFT_LAST_APPLIED, label, 0);

        counter_metric_touch!(RAFT_WRITE_REQUESTS_TOTAL, RaftLabel { machine: shard.clone() });
        counter_metric_touch!(RAFT_WRITE_SUCCESS_TOTAL, RaftLabel { machine: shard.clone() });
        counter_metric_touch!(RAFT_WRITE_FAILURES_TOTAL, RaftLabel { machine: shard.clone() });

        histogram_metric_touch!(RAFT_WRITE_DURATION, RaftLabel { machine: shard.clone() });
        histogram_metric_touch!(RAFT_APPLY_BATCH_DURATION, RaftLabel { machine: shard.clone() });
        histogram_metric_touch!(RAFT_LOG_APPEND_BATCH_DURATION, RaftLabel { machine: shard.clone() });

        for rpc_type in &["append_entries", "install_snapshot", "vote"] {
            let label = RaftRpcLabel { machine: shard.clone(), rpc_type: rpc_type.to_string() };
            counter_metric_touch!(RAFT_RPC_REQUESTS_TOTAL, label);
            let label = RaftRpcLabel { machine: shard.clone(), rpc_type: rpc_type.to_string() };
            counter_metric_touch!(RAFT_RPC_SUCCESS_TOTAL, label);
            let label = RaftRpcLabel { machine: shard.clone(), rpc_type: rpc_type.to_string() };
            counter_metric_touch!(RAFT_RPC_FAILURES_TOTAL, label);
            let label = RaftRpcLabel { machine: shard.clone(), rpc_type: rpc_type.to_string() };
            histogram_metric_touch!(RAFT_RPC_DURATION, label);
        }
    }
}

pub fn record_log_append_batch_duration(machine: &str, duration_ms: f64) {
    let label = RaftLabel {
        machine: machine.to_string(),
    };
    histogram_metric_observe!(RAFT_LOG_APPEND_BATCH_DURATION, duration_ms, label);
}

pub fn record_apply_batch_duration(machine: &str, duration_ms: f64) {
    let label = RaftLabel {
        machine: machine.to_string(),
    };
    histogram_metric_observe!(RAFT_APPLY_BATCH_DURATION, duration_ms, label);
}

pub fn record_raft_apply_lag(machine: &str, last_log: u64, last_applied: u64) {
    let label = RaftLabel {
        machine: machine.to_string(),
    };
    let lag = last_log.saturating_sub(last_applied) as i64;
    gauge_metric_set!(RAFT_APPLY_LAG, label, lag);
    let label = RaftLabel {
        machine: machine.to_string(),
    };
    gauge_metric_set!(RAFT_LAST_LOG_INDEX, label, last_log as i64);
    let label = RaftLabel {
        machine: machine.to_string(),
    };
    gauge_metric_set!(RAFT_LAST_APPLIED, label, last_applied as i64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raft_label_equality() {
        let label1 = RaftLabel {
            machine: "metadata".to_string(),
        };
        let label2 = RaftLabel {
            machine: "metadata".to_string(),
        };
        let label3 = RaftLabel {
            machine: "offset".to_string(),
        };

        assert_eq!(label1, label2);
        assert_ne!(label1, label3);
    }

    #[test]
    fn test_raft_metrics() {
        record_write_request("metadata");
        record_write_success("metadata");
        record_write_failure("mqtt");
        record_write_duration("offset", 12.5);
    }

    #[test]
    fn test_raft_metrics_encode() {
        use crate::core::server::dump_metrics;

        record_write_request("metadata");
        record_write_request("offset");
        record_write_success("metadata");
        record_write_duration("metadata", 10.0);

        let output = dump_metrics();
        println!("=== Prometheus Output ===");
        for line in output.lines() {
            if line.contains("raft_write") {
                println!("{}", line);
            }
        }
        println!("=== End ===");

        assert!(
            output.contains("raft_write_requests"),
            "Counter metric raft_write_requests not found in output! Full output:\n{}",
            output
        );
    }

    #[test]
    fn test_raft_rpc_label_equality() {
        let label1 = RaftRpcLabel {
            machine: "metadata".to_string(),
            rpc_type: "append_entries".to_string(),
        };
        let label2 = RaftRpcLabel {
            machine: "metadata".to_string(),
            rpc_type: "append_entries".to_string(),
        };
        let label3 = RaftRpcLabel {
            machine: "offset".to_string(),
            rpc_type: "vote".to_string(),
        };

        assert_eq!(label1, label2);
        assert_ne!(label1, label3);
    }

    #[test]
    fn test_raft_rpc_metrics() {
        record_rpc_request("metadata", "append_entries");
        record_rpc_success("metadata", "append_entries");
        record_rpc_failure("offset", "vote");
        record_rpc_duration("mqtt", "install_snapshot", 25.8);
    }
}
