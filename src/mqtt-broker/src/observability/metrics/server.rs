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
struct LabelType {
    label: String,
    r#type: String,
}

common_base::register_gauge_metric!(
    BROKER_NETWORK_QUEUE_NUM,
    "network_queue_num",
    "broker network queue num",
    LabelType
);

pub fn metrics_request_queue(label: &str, len: usize) {
    let label_type = LabelType {
        label: label.to_string(),
        r#type: "request".to_string(),
    };
    common_base::gauge_metric_inc_by!(BROKER_NETWORK_QUEUE_NUM, label_type, len as i64);
}

pub fn metrics_response_queue(label: &str, len: usize) {
    let label_type = LabelType {
        label: label.to_string(),
        r#type: "response".to_string(),
    };
    common_base::gauge_metric_inc_by!(BROKER_NETWORK_QUEUE_NUM, label_type, len as i64);
}
