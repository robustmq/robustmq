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
    counter_metric_inc, counter_metric_touch, histogram_metric_observe, histogram_metric_touch,
    register_counter_metric, register_histogram_metric_ms_with_default_buckets,
};
use prometheus_client::encoding::EncodeLabelSet;

// ── Labels ──────────────────────────────────────────────────────────────────

/// `operation` identifies the storage operation being measured.
#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct StorageEngineLabel {
    pub operation: &'static str,
}

// ── Metrics ─────────────────────────────────────────────────────────────────

register_counter_metric!(
    STORAGE_ENGINE_OPS_TOTAL,
    "storage_engine_ops",
    "Total number of storage engine operations",
    StorageEngineLabel
);

register_counter_metric!(
    STORAGE_ENGINE_OPS_FAIL_TOTAL,
    "storage_engine_ops_fail",
    "Total number of failed storage engine operations",
    StorageEngineLabel
);

register_histogram_metric_ms_with_default_buckets!(
    STORAGE_ENGINE_OPS_DURATION_MS,
    "storage_engine_ops_duration_ms",
    "Duration of storage engine operations in milliseconds",
    StorageEngineLabel
);

// ── Public API ──────────────────────────────────────────────────────────────

pub fn record_storage_engine_ops(operation: &'static str) {
    let l = StorageEngineLabel { operation };
    counter_metric_inc!(STORAGE_ENGINE_OPS_TOTAL, l);
}

pub fn record_storage_engine_ops_fail(operation: &'static str) {
    let l = StorageEngineLabel { operation };
    counter_metric_inc!(STORAGE_ENGINE_OPS_FAIL_TOTAL, l);
}

pub fn record_storage_engine_ops_duration(operation: &'static str, duration_ms: f64) {
    let l = StorageEngineLabel { operation };
    histogram_metric_observe!(STORAGE_ENGINE_OPS_DURATION_MS, duration_ms, l);
}

pub fn init() {
    for op in [
        "write",
        "read_offset",
        "read_key",
        "read_tag",
        "create_shard",
        "delete_shard",
        "delete_by_key",
        "delete_by_offset",
        "get_offset_by_timestamp",
        "get_offset_by_group",
        "commit_offset",
        "delete_records_before",
    ] {
        counter_metric_touch!(
            STORAGE_ENGINE_OPS_TOTAL,
            StorageEngineLabel { operation: op }
        );
        counter_metric_touch!(
            STORAGE_ENGINE_OPS_FAIL_TOTAL,
            StorageEngineLabel { operation: op }
        );
        histogram_metric_touch!(
            STORAGE_ENGINE_OPS_DURATION_MS,
            StorageEngineLabel { operation: op }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_storage_operations_and_failures() {
        record_storage_engine_ops("coverage");
        record_storage_engine_ops_fail("coverage");
        record_storage_engine_ops_duration("coverage", 12.5);

        let label = StorageEngineLabel {
            operation: "coverage",
        };
        assert_eq!(
            STORAGE_ENGINE_OPS_TOTAL
                .read()
                .unwrap()
                .get(&label)
                .unwrap()
                .get(),
            1
        );
        assert_eq!(
            STORAGE_ENGINE_OPS_FAIL_TOTAL
                .read()
                .unwrap()
                .get(&label)
                .unwrap()
                .get(),
            1
        );
    }

    #[test]
    fn init_preregisters_all_storage_operations() {
        init();

        for operation in [
            "write",
            "read_offset",
            "commit_offset",
            "delete_records_before",
        ] {
            let label = StorageEngineLabel { operation };
            assert!(STORAGE_ENGINE_OPS_TOTAL
                .read()
                .unwrap()
                .get(&label)
                .is_some());
            assert!(STORAGE_ENGINE_OPS_FAIL_TOTAL
                .read()
                .unwrap()
                .get(&label)
                .is_some());
            assert!(STORAGE_ENGINE_OPS_DURATION_MS
                .read()
                .unwrap()
                .get(&label)
                .is_some());
        }
    }
}
