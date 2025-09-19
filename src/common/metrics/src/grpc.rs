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
    gauge_metric_inc, histogram_metric_observe, meta::raft::MetricsLabel, register_counter_metric,
    register_histogram_metric,
};

register_counter_metric!(
    GRPC_REQUEST_NUM,
    "grpc_request_num",
    "Number of calls to the grpc request",
    MetricsLabel
);

register_histogram_metric!(
    GRPC_REQUEST_TOTAL_MS,
    "grpc_request_total_ms",
    "TotalMs of calls to the grpc request",
    MetricsLabel,
    1.0,
    2.0,
    10
);

pub fn metrics_grpc_request_incr(service: &str, path: &str) {
    let label = MetricsLabel {
        grpc_service: service.to_string(),
        grpc_path: path.to_string(),
        ..Default::default()
    };
    gauge_metric_inc!(GRPC_REQUEST_NUM, label)
}

pub fn metrics_grpc_request_ms(service: &str, path: &str, ms: f64) {
    let label = MetricsLabel {
        grpc_service: service.to_string(),
        grpc_path: path.to_string(),
        ..Default::default()
    };

    histogram_metric_observe!(GRPC_REQUEST_TOTAL_MS, ms, label)
}
