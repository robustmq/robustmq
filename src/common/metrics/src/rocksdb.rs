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
pub struct RocksdbLabel {
    source: String,
    operation: String,
}

register_histogram_metric_ms_with_default_buckets!(
    ROCKSDB_OPERATION_MS,
    "rocksdb_operation_ms",
    "The duration of rocksdb operations",
    RocksdbLabel
);

register_counter_metric!(
    ROCKSDB_OPERATION_COUNT,
    "rocksdb_operation_count",
    "The count of rocksdb operations",
    RocksdbLabel
);

pub fn metrics_rocksdb_save_ms(source: &str, ms: f64) {
    let label = RocksdbLabel {
        source: source.to_string(),
        operation: "save".to_string(),
    };
    histogram_metric_observe!(ROCKSDB_OPERATION_MS, ms, label);
    let count_label = RocksdbLabel {
        source: source.to_string(),
        operation: "save".to_string(),
    };
    counter_metric_inc!(ROCKSDB_OPERATION_COUNT, count_label);
}

pub fn metrics_rocksdb_get_ms(source: &str, ms: f64) {
    let label = RocksdbLabel {
        source: source.to_string(),
        operation: "get".to_string(),
    };
    histogram_metric_observe!(ROCKSDB_OPERATION_MS, ms, label);
    let count_label = RocksdbLabel {
        source: source.to_string(),
        operation: "get".to_string(),
    };
    counter_metric_inc!(ROCKSDB_OPERATION_COUNT, count_label);
}

pub fn metrics_rocksdb_exist_ms(source: &str, ms: f64) {
    let label = RocksdbLabel {
        source: source.to_string(),
        operation: "exist".to_string(),
    };
    histogram_metric_observe!(ROCKSDB_OPERATION_MS, ms, label);
    let count_label = RocksdbLabel {
        source: source.to_string(),
        operation: "exist".to_string(),
    };
    counter_metric_inc!(ROCKSDB_OPERATION_COUNT, count_label);
}

pub fn metrics_rocksdb_delete_ms(source: &str, ms: f64) {
    let label = RocksdbLabel {
        source: source.to_string(),
        operation: "delete".to_string(),
    };
    histogram_metric_observe!(ROCKSDB_OPERATION_MS, ms, label);
    let count_label = RocksdbLabel {
        source: source.to_string(),
        operation: "delete".to_string(),
    };
    counter_metric_inc!(ROCKSDB_OPERATION_COUNT, count_label);
}

pub fn metrics_rocksdb_delete_range_ms(source: &str, ms: f64) {
    let label = RocksdbLabel {
        source: source.to_string(),
        operation: "delete_range".to_string(),
    };
    histogram_metric_observe!(ROCKSDB_OPERATION_MS, ms, label);
    let count_label = RocksdbLabel {
        source: source.to_string(),
        operation: "delete_range".to_string(),
    };
    counter_metric_inc!(ROCKSDB_OPERATION_COUNT, count_label);
}

pub fn metrics_rocksdb_list_ms(source: &str, ms: f64) {
    let label = RocksdbLabel {
        source: source.to_string(),
        operation: "list".to_string(),
    };
    histogram_metric_observe!(ROCKSDB_OPERATION_MS, ms, label);
    let count_label = RocksdbLabel {
        source: source.to_string(),
        operation: "list".to_string(),
    };
    counter_metric_inc!(ROCKSDB_OPERATION_COUNT, count_label);
}
