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

use prometheus_client::encoding::EncodeLabelSet;

use crate::{
    counter_metric_inc, gauge_metric_inc, histogram_metric_observe, register_counter_metric,
    register_histogram_metric,
};

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct MetricsLabel {
    pub grpc_service: String,
    pub grpc_path: String,
    pub raft_storage_type: String,
}

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq, Default)]
pub struct RocksDBLabels {
    pub resource_type: String,

    // TODO: (shyunny) Consider adding an enum for the action instead of accepting the passed parameter in the code?
    pub action: String,
}

impl RocksDBLabels {
    pub fn snapshot(action: &str) -> Self {
        RocksDBLabels {
            resource_type: String::from("snapshot"),
            action: action.to_string(),
        }
    }

    pub fn log(action: &str) -> Self {
        RocksDBLabels {
            resource_type: String::from("log"),
            action: action.to_string(),
        }
    }
}

register_counter_metric!(
    RAFT_STORAGE_TOTAL_NUM,
    "raft_storage_num",
    "Total number of calls to the raft storage",
    MetricsLabel
);

register_counter_metric!(
    RAFT_STORAGE_ERROR_NUM,
    "raft_storage_error_num",
    "Error number of calls to the raft storage",
    MetricsLabel
);

register_histogram_metric!(
    RAFT_STORAGE_TOTAL_MS,
    "raft_storage_total_ms",
    "TotalMs of calls to the raft storage",
    MetricsLabel,
    0.1,
    2.0,
    12
);

register_counter_metric!(
    ROCKSDB_STORAGE_NUM,
    "rocksdb_storage_num",
    "Total number of calls to the rocksdb storage",
    RocksDBLabels
);

register_counter_metric!(
    ROCKSDB_STORAGE_ERROR_NUM,
    "rocksdb_storage_error_num",
    "Error number of calls to the rocksdb storage",
    RocksDBLabels
);

register_histogram_metric!(
    ROCKSDB_STORAGE_TOTAL_MS,
    "rocksdb_storage_total_ms",
    "TotalMs of calls to the rocksdb storage",
    RocksDBLabels,
    0.1,
    2.0,
    12
);

pub fn metrics_raft_storage_total_incr(storage_type: String) {
    let label = MetricsLabel {
        raft_storage_type: storage_type,
        ..Default::default()
    };
    gauge_metric_inc!(RAFT_STORAGE_TOTAL_NUM, label)
}

pub fn metrics_raft_storage_error_incr(storage_type: String) {
    let label = MetricsLabel {
        raft_storage_type: storage_type,
        ..Default::default()
    };
    gauge_metric_inc!(RAFT_STORAGE_ERROR_NUM, label)
}

pub fn metrics_raft_storage_total_ms(storage_type: String, ms: f64) {
    let label = MetricsLabel {
        raft_storage_type: storage_type,
        ..Default::default()
    };

    histogram_metric_observe!(RAFT_STORAGE_TOTAL_MS, ms, label)
}

pub fn metrics_rocksdb_storage_total_inc(labels: RocksDBLabels) {
    counter_metric_inc!(ROCKSDB_STORAGE_NUM, labels)
}

pub fn metrics_rocksdb_storage_err_inc(labels: RocksDBLabels) {
    counter_metric_inc!(ROCKSDB_STORAGE_ERROR_NUM, labels)
}

pub fn metrics_rocksdb_stroge_total_ms(labels: RocksDBLabels, ms: f64) {
    histogram_metric_observe!(ROCKSDB_STORAGE_TOTAL_MS, ms, labels)
}
