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
use crate::common::packet::RequestPackage;
use crate::{command::ArcCommandAdapter, common::channel::RequestChannel};
use common_base::tools::now_mills;
use common_metrics::network::metrics_request_queue_size;
use metadata_struct::connection::NetworkConnectionType;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;
use tracing::{debug, error};

pub async fn handler_process(
    handler_process_num: usize,
    mut request_queue_rx: Receiver<RequestPackage>,
    connection_manager: Arc<ConnectionManager>,
    command: ArcCommandAdapter,
    request_channel: Arc<RequestChannel>,
    network_type: NetworkConnectionType,
    stop_sx: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        handler_child_process(
            handler_process_num,
            connection_manager,
            request_channel.clone(),
            command,
            network_type.clone(),
            stop_sx.clone(),
        );

        let mut stop_rx = stop_sx.subscribe();
        let mut process_handler_seq = 0;

        loop {
            select! {
                val = stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            debug!("{}","Server handler thread stopped successfully.");
                            break;
                        }
                    }
                },
                val = request_queue_rx.recv()=>{
                    if let Some(packet) = val{
                        let mut sleep_ms = 0;
                         metrics_request_queue_size("total", request_queue_rx.len());

                        // Try to deliver the request packet to the child handler until it is delivered successfully.
                        // Because some request queues may be full or abnormal, the request packets can be delivered to other child handlers.
                        loop{
                            process_handler_seq += 1;
                            if let Some(handler_sx) = request_channel.get_available_handler(&network_type, process_handler_seq){
                                if handler_sx.try_send(packet.clone()).is_ok(){
                                    break;
                                }
                            }else{
                                // In exceptional cases, if no available child handler can be found, the request packet is dropped.
                                // If the client does not receive a return packet, it will retry the request.
                                // Rely on repeated requests from the client to ensure that the request will eventually be processed successfully.
                                error!("{}","Handler child thread, no request packet processing thread available");
                            }

                            sleep_ms += 2;
                            sleep(Duration::from_millis(sleep_ms)).await;
                        }

                    }
                }
            }
        }
    });
}

fn handler_child_process(
    handler_process_num: usize,
    connection_manager: Arc<ConnectionManager>,
    request_channel: Arc<RequestChannel>,
    command: ArcCommandAdapter,
    network_type: NetworkConnectionType,
    stop_sx: broadcast::Sender<bool>,
) {
    for index in 1..=handler_process_num {
        let mut child_process_rx =
            request_channel.create_handler_child_channel(&network_type, index);
        let raw_connect_manager = connection_manager.clone();
        let request_channel = request_channel.clone();
        let raw_command = command.clone();
        let mut raw_stop_rx = stop_sx.subscribe();

        let raw_network_type = network_type.clone();
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
                        let label = format!("handler-{index}");
                        metrics_request_queue_size(&label, child_process_rx.len());
                        let out_queue_time = now_mills();
                        if let Some(packet) = val{
                            if let Some(connect) = raw_connect_manager.get_connect(packet.connection_id) {

                                let response_data = raw_command
                                    .apply(connect, packet.addr, packet.packet)
                                    .await;

                                if let Some(mut resp) = response_data {
                                    resp.out_queue_ms = out_queue_time;
                                    resp.receive_ms = packet.receive_ms;
                                    resp.end_handler_ms = now_mills();
                                    request_channel.send_response_channel(&raw_network_type, resp).await;
                                } else {
                                    debug!("{}","No backpacking is required for this request");
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
