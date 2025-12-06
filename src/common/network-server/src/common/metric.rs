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

use crate::common::packet::{RequestPackage, ResponsePackage};
use common_base::tools::now_millis;
use common_metrics::network::{
    metrics_request_handler_ms, metrics_request_not_response_total_ms, metrics_request_queue_ms,
    metrics_request_response_ms, metrics_request_response_queue_ms, metrics_request_total_ms,
};
use metadata_struct::connection::NetworkConnectionType;
use tracing::info;

pub fn record_packet_handler_info_no_response(
    request_packet: &RequestPackage,
    out_handler_queue_ms: u128,
    end_handler_ms: u128,
    packet_name: String,
) {
    let end_ms = now_millis();
    let handler_queue_ms = out_handler_queue_ms - request_packet.receive_ms;
    let handler_ms = end_handler_ms - out_handler_queue_ms;
    let total_ms = now_millis() - request_packet.receive_ms;

    // request queue ms
    metrics_request_queue_ms(&NetworkConnectionType::Tcp, handler_queue_ms as f64);

    // handler ms
    metrics_request_handler_ms(&NetworkConnectionType::Tcp, handler_ms as f64);

    // total ms
    metrics_request_total_ms(&NetworkConnectionType::Tcp, total_ms as f64);

    if request_packet.receive_ms > 0 && is_record_ms_log(total_ms) {
        info!(
            "packet:{}, total_ms:{}, receive_ms:{}, handler_queue_ms:{}, handler_ms:{}, end_ms:{}",
            packet_name, total_ms, request_packet.receive_ms, handler_queue_ms, handler_ms, end_ms
        );
    }
}

pub fn record_packet_handler_info_by_response(
    network_type: &NetworkConnectionType,
    response_package: &ResponsePackage,
    out_response_queue_ms: u128,
) {
    let end_ms = now_millis();
    let request_queue_ms = response_package.out_queue_ms - response_package.receive_ms;
    let handler_ms = response_package.end_handler_ms - response_package.out_queue_ms;
    let response_queue_ms = out_response_queue_ms - response_package.end_handler_ms;
    let response_ms = end_ms - out_response_queue_ms;
    let no_response_total_ms = out_response_queue_ms - response_package.receive_ms;
    let total_ms = end_ms - response_package.receive_ms;

    // request queue ms
    metrics_request_queue_ms(network_type, request_queue_ms as f64);

    // handler ms
    metrics_request_handler_ms(network_type, handler_ms as f64);

    // response queue ms
    metrics_request_response_queue_ms(network_type, response_queue_ms as f64);

    // response ms
    metrics_request_response_ms(network_type, response_ms as f64);

    // not response total ms
    metrics_request_not_response_total_ms(network_type, no_response_total_ms as f64);

    // total ms
    metrics_request_total_ms(network_type, total_ms as f64);

    if response_package.receive_ms > 0 && is_record_ms_log(total_ms) {
        info!("packet:{},receive_ms:{}, total_ms:{},  handler_queue_ms:{}, handler_ms:{}, response_queue_ms:{}, response_ms:{}, end_ms:{}",
                    response_package.request_packet,  response_package.receive_ms, total_ms, request_queue_ms, handler_ms, response_queue_ms, response_ms, end_ms);
    }
}

fn is_record_ms_log(total: u128) -> bool {
    total >= 500000
}
