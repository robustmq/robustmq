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

use crate::handler::cache::CacheManager;
use crate::handler::connection::disconnect_connection;
use crate::observability::metrics::server::{
    metrics_response_queue_size, record_response_and_total_ms,
};
use crate::server::common::channel::RequestChannel;
use crate::server::common::connection::NetworkConnectionType;
use crate::server::common::connection_manager::ConnectionManager;
use crate::server::common::metric::record_packet_handler_info_by_response;
use crate::server::common::packet::ResponsePackage;
use crate::subscribe::manager::SubscribeManager;
use common_base::tools::now_mills;
use grpc_clients::pool::ClientPool;
use protocol::mqtt::codec::MqttPacketWrapper;
use protocol::mqtt::common::MqttPacket;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;
use tracing::{debug, error};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn response_process(
    response_process_num: usize,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    mut response_queue_rx: Receiver<ResponsePackage>,
    client_pool: Arc<ClientPool>,
    request_channel: Arc<RequestChannel>,
    network_type: NetworkConnectionType,
    stop_sx: broadcast::Sender<bool>,
) {
    let mut stop_rx = stop_sx.subscribe();
    tokio::spawn(async move {
        response_child_process(
            response_process_num,
            request_channel.clone(),
            connection_manager,
            cache_manager,
            subscribe_manager,
            client_pool,
            network_type.clone(),
            stop_sx.clone(),
        );

        let mut response_process_seq = 0;
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
                        let mut sleep_ms = 0;
                        metrics_response_queue_size("total", response_queue_rx.len());

                        loop {
                            response_process_seq += 1;
                            if let Some(handler_sx) = request_channel.get_available_response(&network_type,response_process_seq){
                                if handler_sx.try_send(packet.clone()).is_ok() {
                                    break;
                                }
                            }else{
                                error!("{}","Response child thread, no request packet processing thread available");
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn response_child_process(
    response_process_num: usize,
    request_channel: Arc<RequestChannel>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_pool: Arc<ClientPool>,
    network_type: NetworkConnectionType,
    stop_sx: broadcast::Sender<bool>,
) {
    for index in 1..=response_process_num {
        let mut response_process_rx =
            request_channel.create_response_child_channel(&network_type, index);
        let mut raw_stop_rx = stop_sx.subscribe();
        let raw_connect_manager = connection_manager.clone();
        let raw_cache_manager = cache_manager.clone();
        let raw_client_pool = client_pool.clone();
        let raw_subscribe_manager = subscribe_manager.clone();
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
                            let label = format!("handler-{index}");
                            metrics_response_queue_size(&label, response_process_rx.len());
                            let mut response_ms = now_mills();
                            if let Some(protocol) =raw_connect_manager.get_connect_protocol(response_package.connection_id)
                                {
                                    let packet_wrapper = MqttPacketWrapper {
                                        protocol_version: protocol.into(),
                                        packet: response_package.packet.clone(),
                                    };

                                    if let Err(e) =  raw_connect_manager.write_tcp_frame(response_package.connection_id, packet_wrapper).await {
                                        error!("{}",e);
                                    };

                                    response_ms = now_mills();
                                    record_response_and_total_ms(&NetworkConnectionType::Tcp,response_package.get_receive_ms(),out_response_queue_ms);
                            }

                            if let MqttPacket::Disconnect(_, _) = response_package.packet {
                                if let Some(connection) = raw_cache_manager.get_connection(response_package.connection_id){
                                    if let Err(e) =  disconnect_connection(
                                        &connection.client_id,
                                        connection.connect_id,
                                        &raw_cache_manager,
                                        &raw_client_pool,
                                        &raw_connect_manager,
                                        &raw_subscribe_manager,
                                        true
                                    ).await{
                                        error!("{}",e);
                                    };
                                }
                            }
                            record_packet_handler_info_by_response(&response_package, out_response_queue_ms, response_ms);
                        }
                    }
                }
            }
        });
    }
}
