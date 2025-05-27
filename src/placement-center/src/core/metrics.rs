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

use crate::route::data::StorageDataType;

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

common_base::register_counter_metric!(
    GRPC_REQUEST_NUM,
    "grpc.request.num",
    "Number of calls to the grpc request",
    MetricsLabel
);

common_base::register_histogram_metric!(
    GRPC_REQUEST_TOTAL_MS,
    "grpc.request.total.ms",
    "TotalMs of calls to the grpc request",
    MetricsLabel
);

common_base::register_counter_metric!(
    RAFT_STORAGE_TOTAL_NUM,
    "raft.storage.num",
    "Total number of calls to the raft storage",
    MetricsLabel
);

common_base::register_counter_metric!(
    RAFT_STORAGE_ERROR_NUM,
    "raft.storage.error.num",
    "Error number of calls to the raft storage",
    MetricsLabel
);

common_base::register_histogram_metric!(
    RAFT_STORAGE_TOTAL_MS,
    "raft.storage.total.ms",
    "TotalMs of calls to the raft storage",
    MetricsLabel
);

common_base::register_counter_metric!(
    ROCKSDB_STORAGE_NUM,
    "rocksdb.storage.num",
    "Total number of calls to the rocksdb storage",
    RocksDBLabels
);

common_base::register_counter_metric!(
    ROCKSDB_STORAGE_ERROR_NUM,
    "rocksdb.storage.error.num",
    "Error number of calls to the rocksdb storage",
    RocksDBLabels
);

common_base::register_histogram_metric!(
    ROCKSDB_STORAGE_TOTAL_MS,
    "rocksdb.storage.total.ms",
    "TotalMs of calls to the rocksdb storage",
    RocksDBLabels
);

pub fn metrics_grpc_request_incr(service: &str, path: &str) {
    let label = MetricsLabel {
        grpc_service: service.to_string(),
        grpc_path: path.to_string(),
        ..Default::default()
    };
    common_base::gauge_metric_inc!(GRPC_REQUEST_NUM, label)
}

pub fn metrics_grpc_request_ms(service: &str, path: &str, ms: f64) {
    let label = MetricsLabel {
        grpc_service: service.to_string(),
        grpc_path: path.to_string(),
        ..Default::default()
    };

    common_base::histogram_metric_observe!(GRPC_REQUEST_TOTAL_MS, ms, label)
}

pub fn metrics_raft_storage_total_incr(storage_type: &StorageDataType) {
    let label = MetricsLabel {
        raft_storage_type: format!("{:?}", storage_type),
        ..Default::default()
    };
    common_base::gauge_metric_inc!(RAFT_STORAGE_TOTAL_NUM, label)
}

pub fn metrics_raft_storage_error_incr(storage_type: &StorageDataType) {
    let label = MetricsLabel {
        raft_storage_type: format!("{:?}", storage_type),
        ..Default::default()
    };
    common_base::gauge_metric_inc!(RAFT_STORAGE_ERROR_NUM, label)
}

pub fn metrics_raft_storage_total_ms(storage_type: &StorageDataType, ms: f64) {
    let label = MetricsLabel {
        raft_storage_type: format!("{:?}", storage_type),
        ..Default::default()
    };

    common_base::histogram_metric_observe!(RAFT_STORAGE_TOTAL_MS, ms, label)
}

pub fn metrics_rocksdb_storage_total_inc(labels: RocksDBLabels) {
    common_base::counter_metric_inc!(ROCKSDB_STORAGE_NUM, labels)
}

pub fn metrics_rocksdb_storage_err_inc(labels: RocksDBLabels) {
    common_base::counter_metric_inc!(ROCKSDB_STORAGE_ERROR_NUM, labels)
}

pub fn metrics_rocksdb_stroge_total_ms(labels: RocksDBLabels, ms: f64) {
    common_base::histogram_metric_observe!(ROCKSDB_STORAGE_TOTAL_MS, ms, labels)
}
