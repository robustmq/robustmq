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

use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::{debug, error};

use crate::core::error::JournalServerError;
use crate::handler::command::Command;
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::{RequestPackage, ResponsePackage};

/// spawn `handler_process_num` threads to process request packets, distribute packets to threads
pub(crate) async fn handler_process(
    handler_process_num: usize,
    mut request_queue_rx: Receiver<RequestPackage>,
    connection_manager: Arc<ConnectionManager>,
    response_queue_sx: Sender<ResponsePackage>,
    stop_sx: broadcast::Sender<bool>,
    command: Command,
) {
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
        let mut process_handler_seq = 1;
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
                        // Try to deliver the request packet to the child handler until it is delivered successfully.
                        // Because some request queues may be full or abnormal, the request packets can be delivered to other child handlers.
                        loop{
                            let seq = if process_handler_seq > child_process_list.len(){
                                1
                            } else {
                                process_handler_seq
                            };

                            if let Some(handler_sx) = child_process_list.get(&seq){
                                match handler_sx.try_send(packet.clone()){
                                    Ok(_) => {
                                        break;
                                    }
                                    Err(err) => error!(
                                        "Failed to try write data to the handler process queue, error message: {:?}",
                                        err
                                    ),
                                }
                                process_handler_seq += 1;
                            }else{
                                // In exceptional cases, if no available child handler can be found, the request packet is dropped.
                                // If the client does not receive a return packet, it will retry the request.
                                // Rely on repeated requests from the client to ensure that the request will eventually be processed successfully.
                                error!("{}","No request packet processing thread available");
                                break;
                            }
                        }

                    }
                }
            }
        }
    });
}

/// each child process consumes the request packets from the request queue, applies the command, and sends the response to the response queue
fn handler_child_process(
    handler_process_num: usize,
    stop_sx: broadcast::Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
    command: Command,
    child_process_list: &mut HashMap<usize, Sender<RequestPackage>>,
    response_queue_sx: Sender<ResponsePackage>,
) {
    for index in 1..=handler_process_num {
        let (child_handler_sx, mut child_process_rx) = mpsc::channel::<RequestPackage>(1000);
        child_process_list.insert(index, child_handler_sx.clone());

        let mut raw_stop_rx = stop_sx.subscribe();
        let raw_connect_manager = connection_manager.clone();
        let raw_response_queue_sx = response_queue_sx.clone();
        let raw_command = command.clone();

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
                            if let Some(connect) = raw_connect_manager.get_connect(packet.connection_id) {
                                if let Some(resp) = raw_command
                                    .apply(raw_connect_manager.clone(), connect, packet.addr, packet.packet)
                                    .await
                                {
                                    let response_package = ResponsePackage::new(packet.connection_id, resp);
                                    match raw_response_queue_sx.send(response_package).await {
                                        Ok(_) => {}
                                        Err(err) => error!(
                                            "Failed to write data to the response queue, error message: {:?}",
                                            err
                                        ),
                                    }
                                } else {
                                    debug!("{}","No backpacking is required for this request");
                                }
                            } else {
                                error!("{}", JournalServerError::NotFoundConnectionInCache(packet.connection_id));
                            }
                        }
                    }
                }
            }
        });
    }
}
