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

use grpc_clients::pool::ClientPool;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::{debug, error};

use crate::core1::cache::CacheManager;
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::ResponsePackage;

/// spawn `response_process_num` threads to process response packets, distribute packets to threads
pub(crate) async fn response_process(
    response_process_num: usize,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    mut response_queue_rx: Receiver<ResponsePackage>,
    client_pool: Arc<ClientPool>,
    stop_sx: broadcast::Sender<bool>,
) {
    let mut stop_rx = stop_sx.subscribe();
    tokio::spawn(async move {
        let mut process_handler: HashMap<usize, Sender<ResponsePackage>> = HashMap::new();
        response_child_process(
            response_process_num,
            &mut process_handler,
            stop_sx.clone(),
            connection_manager,
            cache_manager,
            client_pool,
        );

        let mut response_process_seq = 1;
        loop {
            select! {
                val = stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            debug!("{}","TCP Server response process thread stopped successfully.");
                            break;
                        }
                    }
                }

                val = response_queue_rx.recv()=>{
                    if let Some(packet) = val{
                        loop{
                            let seq = if response_process_seq > process_handler.len(){
                                1
                            } else {
                                response_process_seq
                            };

                            if let Some(handler_sx) = process_handler.get(&seq){
                                match handler_sx.try_send(packet.clone()){
                                    Ok(_) => {
                                        break;
                                    }
                                    Err(err) => error!(
                                        "Failed to write data to the quic response process queue, error message: {:?}",
                                        err
                                    ),
                                }
                                response_process_seq += 1;
                            }else{
                                error!("{}","No request packet processing thread available");
                            }
                        }
                    }
                }
            }
        }
    });
}

/// each child process consume the response packet from the response queue, and write the packet to the corresponding tcp write stream
pub(crate) fn response_child_process(
    response_process_num: usize,
    process_handler: &mut HashMap<usize, Sender<ResponsePackage>>,
    stop_sx: broadcast::Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
    _cache_manager: Arc<CacheManager>,
    _client_pool: Arc<ClientPool>,
) {
    for index in 1..=response_process_num {
        let (response_process_sx, mut response_process_rx) = mpsc::channel::<ResponsePackage>(100);
        process_handler.insert(index, response_process_sx.clone());

        let mut raw_stop_rx = stop_sx.subscribe();
        let raw_connect_manager = connection_manager.clone();
        tokio::spawn(async move {
            debug!("TCP Server response process thread {index} start successfully.");

            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                debug!("TCP Server response process thread {index} stopped successfully.");
                                break;
                            }
                        }
                    },
                    val = response_process_rx.recv()=>{
                        if let Some(response_package) = val{
                            match raw_connect_manager.write_tcp_frame(response_package.connection_id, response_package.packet).await{
                                Ok(()) => {},
                                Err(e) => {
                                    error!("{}",e);
                                    raw_connect_manager.close_connect(response_package.connection_id).await;
                                }
                            }

                        }
                    }
                }
            }
        });
    }
}
