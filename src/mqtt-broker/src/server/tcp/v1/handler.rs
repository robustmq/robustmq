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

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::handler::command::Command;
use crate::observability::metrics::server::metrics_request_queue_size;
use crate::server::connection::calc_child_channel_index;
use crate::server::connection_manager::ConnectionManager;
use crate::server::metric::record_packet_handler_info_no_response;
use crate::server::packet::{RequestPackage, ResponsePackage};
use common_base::tools::now_mills;
use protocol::mqtt::common::mqtt_packet_to_string;
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::sleep;
use tracing::{debug, error, info};

pub(crate) async fn handler_process<S>(
    handler_process_num: usize,
    mut request_queue_rx: Receiver<RequestPackage>,
    connection_manager: Arc<ConnectionManager>,
    response_queue_sx: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
    command: Command<S>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    tokio::spawn(async move {
        let mut child_process_list: HashMap<usize, Sender<RequestPackage>> = HashMap::new();
        handler_child_process(
            handler_process_num,
            stop_sx.clone(),
            connection_manager,
            command,
            &mut child_process_list,
            response_queue_sx,
        );

        let mut stop_rx = stop_sx.subscribe();
        let mut process_handler_seq = 0;

        loop {
            select! {
                val = stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            debug!("{}","TCP Server handler thread stopped successfully.");
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
                            let seq = calc_child_channel_index(process_handler_seq,child_process_list.len());
                            if let Some(handler_sx) = child_process_list.get(&seq){
                                if handler_sx.try_send(packet.clone()).is_ok(){
                                    break;
                                }
                                sleep_ms += 1;
                                sleep(Duration::from_millis(sleep_ms)).await;
                            }else{
                                // In exceptional cases, if no available child handler can be found, the request packet is dropped.
                                // If the client does not receive a return packet, it will retry the request.
                                // Rely on repeated requests from the client to ensure that the request will eventually be processed successfully.
                                error!("{}","Handler child thread, no request packet processing thread available");
                                break;
                            }
                        }

                    }
                }
            }
        }
    });
}

fn handler_child_process<S>(
    handler_process_num: usize,
    stop_sx: broadcast::Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
    command: Command<S>,
    child_process_list: &mut HashMap<usize, Sender<RequestPackage>>,
    response_queue_sx: Sender<ResponsePackage>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    for index in 1..=handler_process_num {
        let (child_handler_sx, mut child_process_rx) = mpsc::channel::<RequestPackage>(1000);
        child_process_list.insert(index, child_handler_sx.clone());

        let mut raw_stop_rx = stop_sx.subscribe();
        let raw_connect_manager = connection_manager.clone();
        let raw_response_queue_sx = response_queue_sx.clone();
        let mut raw_command = command.clone();

        tokio::spawn(async move {
            debug!(
                "TCP Server handler process thread {} start successfully.",
                index
            );
            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                debug!("TCP Server handler process thread {} stopped successfully.",index);
                                break;
                            }
                        }
                    },
                    val = child_process_rx.recv()=>{
                        if let Some(packet) = val{
                            let label = format!("handler-{}",index);
                            metrics_request_queue_size(&label, child_process_rx.len());
                            if let Some(connect) = raw_connect_manager.get_connect(packet.connection_id) {
                                let out_handler_queue_ms = now_mills();

                                let response_data = raw_command
                                    .apply(&raw_connect_manager, &connect, &packet.addr, &packet.packet)
                                    .await;
                                let end_handler_ms = now_mills();

                                if let Some(resp) = response_data {
                                    let response_package = ResponsePackage::new(packet.connection_id, resp,packet.receive_ms,
                                                out_handler_queue_ms,end_handler_ms, mqtt_packet_to_string(&packet.packet));
                                    if let Err(err) = raw_response_queue_sx.send(response_package).await {
                                        error!(
                                            "Failed to write data to the response queue, error message: {:?}",
                                            err
                                        );
                                    }
                                } else {
                                    record_packet_handler_info_no_response(&packet, out_handler_queue_ms, end_handler_ms, mqtt_packet_to_string(&packet.packet));
                                    info!("{}","No backpacking is required for this request");
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
