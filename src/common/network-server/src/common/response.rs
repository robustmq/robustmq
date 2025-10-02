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

use crate::common::connection_manager::ConnectionManager;
use crate::common::packet::{build_mqtt_packet_wrapper, ResponsePackage};
use crate::common::tool::calc_resp_channel_len;
use crate::common::{channel::RequestChannel, metric::record_packet_handler_info_by_response};
use common_base::error::not_record_error;
use common_base::tools::now_mills;
use common_metrics::network::{metrics_response_queue_size, record_response_and_total_ms};
use metadata_struct::connection::NetworkConnectionType;
use protocol::robust::RobustMQPacket;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error};

#[derive(Clone)]
pub struct ResponseChildProcessContext {
    pub response_process_num: usize,
    pub request_channel: Arc<RequestChannel>,
    pub connection_manager: Arc<ConnectionManager>,
    pub network_type: NetworkConnectionType,
    pub stop_sx: broadcast::Sender<bool>,
}

pub fn response_process(context: ResponseChildProcessContext) {
    for index in 1..=context.response_process_num {
        let request_channel = context.request_channel.clone();
        let mut response_process_rx =
            request_channel.create_response_child_channel(&context.network_type, index);
        let mut raw_stop_rx = context.stop_sx.subscribe();
        let raw_connect_manager = context.connection_manager.clone();
        let network_type = context.network_type.clone();
        tokio::spawn(async move {
            debug!("Server response process thread {index} start successfully.");
            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                debug!("Server response process thread {index} stopped successfully.");
                                break;
                            }
                        }
                    },
                    val = response_process_rx.recv()=>{
                        if let Some(response_package) = val{
                            let out_response_queue_ms = now_mills();
                            record_response_channel_metrics(&response_process_rx,index, request_channel.channel_size);

                            let mut response_ms = now_mills();
                            if let Some(protocol) = raw_connect_manager.get_connect_protocol(response_package.connection_id){
                                let packet_wrapper = match response_package.packet.clone(){
                                    RobustMQPacket::MQTT(packet) => {
                                        build_mqtt_packet_wrapper(protocol, packet)
                                    }
                                    RobustMQPacket::KAFKA(_packet) => {
                                        // todo
                                        return;
                                    }
                                };

                                match &network_type.clone() {
                                    NetworkConnectionType::Tcp | NetworkConnectionType::Tls | NetworkConnectionType::WebSocket |  NetworkConnectionType::WebSockets => {
                                         if let Err(e) =  raw_connect_manager.write_tcp_frame(response_package.connection_id, packet_wrapper).await {
                                            if not_record_error(&e.to_string()){
                                                continue;
                                            }
                                            error!("{}",e);
                                         };
                                    }
                                    NetworkConnectionType::QUIC => {
                                        if let Err(e) =  raw_connect_manager.write_quic_frame(response_package.connection_id, packet_wrapper).await {
                                            if not_record_error(&e.to_string()){
                                                continue;
                                            }
                                            error!("{}",e);
                                         };
                                    }
                                }
                                response_ms = now_mills();
                                record_response_and_total_ms(&network_type.clone(),response_package.get_receive_ms(),out_response_queue_ms);
                            }
                            record_packet_handler_info_by_response(&response_package, out_response_queue_ms, response_ms);
                        }
                    }
                }
            }
        });
    }
}

fn record_response_channel_metrics(
    recv: &Receiver<ResponsePackage>,
    index: usize,
    channel_size: usize,
) {
    let label = format!("handler-{index}");
    let (block_size, remaining_size, use_size) = calc_resp_channel_len(recv, channel_size);
    metrics_response_queue_size(&label, block_size, use_size, remaining_size);
}
