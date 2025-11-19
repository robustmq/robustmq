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
    "raft_write_requests_total",
    "Total number of write requests",
    RaftLabel
);

register_counter_metric!(
    RAFT_WRITE_SUCCESS_TOTAL,
    "raft_write_success_total",
    "Total number of successful writes",
    RaftLabel
);

register_counter_metric!(
    RAFT_WRITE_FAILURES_TOTAL,
    "raft_write_failures_total",
    "Total number of failed writes",
    RaftLabel
);

register_histogram_metric_ms_with_default_buckets!(
    RAFT_WRITE_DURATION,
    "raft_write_duration_ms",
    "Duration of write operations in milliseconds",
    RaftLabel
);

register_counter_metric!(
    RAFT_RPC_REQUESTS_TOTAL,
    "raft_rpc_requests_total",
    "Total number of RPC requests",
    RaftRpcLabel
);

register_counter_metric!(
    RAFT_RPC_SUCCESS_TOTAL,
    "raft_rpc_success_total",
    "Total number of successful RPC requests",
    RaftRpcLabel
);

register_counter_metric!(
    RAFT_RPC_FAILURES_TOTAL,
    "raft_rpc_failures_total",
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
