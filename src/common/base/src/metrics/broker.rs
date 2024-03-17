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

use super::{SERVER_LABLE_MQTT, SERVER_LABLE_GRPC, SERVER_LABLE_HTTP};

const SERVER_MODULE: &str = "server_module";

lazy_static! {
    static ref BROKER_STATUS: IntGaugeVec =
        register_int_gauge_vec!("broker_status", "broker status", &[SERVER_MODULE,]).unwrap();
}

pub fn metrics_mqtt4_broker_running() {
    BROKER_STATUS.with_label_values(&[SERVER_LABLE_MQTT]).set(1);
}

pub fn metrics_grpc_broker_running() {
    BROKER_STATUS.with_label_values(&[SERVER_LABLE_GRPC]).set(1);
}

pub fn metrics_http_broker_running() {
    BROKER_STATUS.with_label_values(&[SERVER_LABLE_HTTP]).set(1);
}
