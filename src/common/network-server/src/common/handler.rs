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
use crate::common::metric::record_packet_handler_info_by_response;
use crate::common::packet::{build_mqtt_packet_wrapper, RequestPackage, ResponsePackage};
use crate::common::tool::calc_req_channel_len;
use crate::{command::ArcCommandAdapter, common::channel::RequestChannel};
use common_base::error::not_record_error;
use common_base::tools::now_millis;
use common_metrics::network::metrics_request_queue_size;
use metadata_struct::connection::NetworkConnectionType;
use protocol::robust::RobustMQPacket;
use std::sync::Arc;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{broadcast, Semaphore};
use tracing::{debug, error};

pub fn handler_process(
    handler_process_num: usize,
    connection_manager: Arc<ConnectionManager>,
    command: ArcCommandAdapter,
    request_channel: Arc<RequestChannel>,
    network_type: NetworkConnectionType,
    stop_sx: broadcast::Sender<bool>,
) {
    for index in 1..=handler_process_num {
        let mut child_process_rx = request_channel.create_handler_channel(&network_type, index);
        let raw_connect_manager = connection_manager.clone();
        let request_channel = request_channel.clone();
        let raw_command = command.clone();
        let mut raw_stop_rx = stop_sx.subscribe();
        let raw_network_type = network_type.clone();

        let semaphore = Arc::new(Semaphore::new(5));
        tokio::spawn(async move {
            debug!(
                "Server handler process thread {} start successfully.",
                index
            );
            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                debug!("Server handler process thread {} stopped successfully.",index);
                                break;
                            }
                        }
                    },
                    val = child_process_rx.recv()=>{
                        let out_queue_time = now_millis();
                        record_request_channel_metrics(&child_process_rx,index,request_channel.channel_size);
                        if let Some(packet) = val{
                            let permit = semaphore.clone().acquire_owned().await.unwrap();
                            let permit_raw_connect_manager = raw_connect_manager.clone();
                            let permit_raw_command = raw_command.clone();
                            let permit_raw_network_type = raw_network_type.clone();
                            tokio::spawn(async move{
                                if let Some(connect) = permit_raw_connect_manager.get_connect(packet.connection_id) {
                                    let response_data = permit_raw_command
                                        .apply(&connect, &packet.addr, &packet.packet)
                                        .await;

                                    if let Some(mut resp) = response_data {
                                        resp.out_queue_ms = out_queue_time;
                                        resp.receive_ms = packet.receive_ms;
                                        resp.end_handler_ms = now_millis();
                                        // permit_request_channel.send_response_packet_to_handler(&permit_raw_network_type, resp).await;
                                        process_response(&permit_raw_connect_manager, &permit_raw_network_type, &resp).await;
                                    } else {
                                        debug!("{}","No backpacking is required for this request");
                                    }
                                }
                                drop(permit);
                            });
                        }
                    }
                }
            }
        });
    }
}

fn record_request_channel_metrics(
    recv: &Receiver<RequestPackage>,
    index: usize,
    channel_size: usize,
) {
    let label = format!("handler-{index}");
    let (block_size, remaining_size, use_size) = calc_req_channel_len(recv, channel_size);
    metrics_request_queue_size(&label, block_size, use_size, remaining_size);
}

async fn process_response(
    connection_manager: &Arc<ConnectionManager>,
    network_type: &NetworkConnectionType,
    response_package: &ResponsePackage,
) {
    let out_response_queue_ms = now_millis();
    if let Some(protocol) = connection_manager.get_connect_protocol(response_package.connection_id)
    {
        let packet_wrapper = match response_package.packet.clone() {
            RobustMQPacket::MQTT(packet) => build_mqtt_packet_wrapper(protocol, packet),
            RobustMQPacket::KAFKA(_packet) => {
                // todo
                return;
            }
        };

        match network_type.clone() {
            NetworkConnectionType::Tcp
            | NetworkConnectionType::Tls
            | NetworkConnectionType::WebSocket
            | NetworkConnectionType::WebSockets => {
                if let Err(e) = connection_manager
                    .write_tcp_frame(response_package.connection_id, packet_wrapper)
                    .await
                {
                    if not_record_error(&e.to_string()) {
                        return;
                    }
                    error!("{}", e);
                };
            }
            NetworkConnectionType::QUIC => {
                if let Err(e) = connection_manager
                    .write_quic_frame(response_package.connection_id, packet_wrapper)
                    .await
                {
                    if not_record_error(&e.to_string()) {
                        return;
                    }
                    error!("{}", e);
                };
            }
        }
        record_packet_handler_info_by_response(
            network_type,
            response_package,
            out_response_queue_ms,
        );
    }
}
