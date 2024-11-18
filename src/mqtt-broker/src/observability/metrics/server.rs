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

use lazy_static::lazy_static;
use prometheus::{register_int_gauge_vec, IntGaugeVec};

use crate::handler::constant::{METRICS_KEY_LABEL_NAME, METRICS_KEY_TYPE_NAME};

lazy_static! {
    static ref BROKER_NETWORK_QUEUE_NUM: IntGaugeVec = register_int_gauge_vec!(
        "network_queue_num",
        "broker network queue num",
        &[METRICS_KEY_LABEL_NAME, METRICS_KEY_TYPE_NAME]
    )
    .unwrap();
}

pub fn metrics_request_queue(label: &str, len: usize) {
    BROKER_NETWORK_QUEUE_NUM
        .with_label_values(&[label, "request"])
        .set(len as i64);
}

pub fn metrics_response_queue(label: &str, len: usize) {
    BROKER_NETWORK_QUEUE_NUM
        .with_label_values(&[label, "response"])
        .set(len as i64);
}
