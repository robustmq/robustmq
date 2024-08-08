// Copyright 2023 RobustMQ Team
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

const METRICS_KEY_MODULE_NAME: &str = "module";
const METRICS_KEY_PROTOCOL_NAME: &str = "protocol";
const METRICS_KEY_TYPE_NAME: &str = "type";

lazy_static! {
    static ref BROKER_PACKET_NUM: IntGaugeVec = register_int_gauge_vec!(
        "broker_packet_num",
        "broker packet num",
        &[
            METRICS_KEY_MODULE_NAME,
            METRICS_KEY_PROTOCOL_NAME,
            METRICS_KEY_TYPE_NAME,
        ]
    )
    .unwrap();
    static ref BROKER_NETWORK_QUEUE_NUM: IntGaugeVec = register_int_gauge_vec!(
        "broker_network_queue_num",
        "broker network queue num",
        &[
            METRICS_KEY_MODULE_NAME,
            METRICS_KEY_PROTOCOL_NAME,
            METRICS_KEY_TYPE_NAME,
        ]
    )
    .unwrap();
    static ref BROKER_TCP_CONNECTION_NUM: IntGaugeVec = register_int_gauge_vec!(
        "broker_connection_num",
        "broker connection num",
        &[METRICS_KEY_MODULE_NAME, METRICS_KEY_PROTOCOL_NAME,]
    )
    .unwrap();
    static ref HEARTBEAT_KEEP_ALIVE_RUN_TIMES: IntGaugeVec = register_int_gauge_vec!(
        "heartbeat_keep_alive_run_info",
        "heartbeat keep alive run info",
        &[METRICS_KEY_MODULE_NAME]
    )
    .unwrap();
}

#[allow(dead_code)]
pub fn metrics_request_packet_incr(lable: &str) {
    BROKER_PACKET_NUM
        .with_label_values(&["broker", lable, "request"])
        .inc();
}

#[allow(dead_code)]
pub fn metrics_response_packet_incr(lable: &str) {
    BROKER_PACKET_NUM
        .with_label_values(&["broker", lable, "response"])
        .inc();
}

pub fn metrics_request_queue(lable: &str, len: i64) {
    BROKER_NETWORK_QUEUE_NUM
        .with_label_values(&["broker", lable, "request"])
        .set(len);
}

pub fn metrics_response_queue(lable: &str, len: i64) {
    BROKER_NETWORK_QUEUE_NUM
        .with_label_values(&["broker", lable, "response"])
        .set(len);
}

#[allow(dead_code)]
pub fn metrics_connection_num(lable: &str, len: i64) {
    BROKER_TCP_CONNECTION_NUM
        .with_label_values(&["broker", lable])
        .set(len);
}

#[allow(dead_code)]
pub fn metrics_heartbeat_keep_alive_run_info(time: u128) {
    HEARTBEAT_KEEP_ALIVE_RUN_TIMES
        .with_label_values(&["broker"])
        .set(time as i64);
}
