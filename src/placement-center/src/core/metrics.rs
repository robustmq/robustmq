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

#[derive(Eq, Hash, Clone, EncodeLabelSet, Debug, PartialEq)]
pub struct GrpcMethodLabel {
    pub method: String,
}

common_base::register_counter_metric!(
    GRPC_REQUEST_NUM,
    "grpc_request_num",
    "Number of calls to the grpc request",
    GrpcMethodLabel
);

pub fn metrics_grpc_request_incr(method: &str) {
    let label = GrpcMethodLabel {
        method: method.to_string(),
    };
    common_base::gauge_metric_inc!(GRPC_REQUEST_NUM, label)
}

pub fn metrics_grpc_request_ms(_: u128) {}
