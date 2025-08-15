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

pub struct ResponseProcessContext {
    pub response_process_num: usize,
    pub connection_manager: Arc<ConnectionManager>,
    pub cache_manager: Arc<CacheManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub response_queue_rx: Receiver<ResponsePackage>,
    pub client_pool: Arc<ClientPool>,
    pub request_channel: Arc<RequestChannel>,
    pub network_type: NetworkConnectionType,
    pub stop_sx: broadcast::Sender<bool>,
}

#[derive(Clone)]
pub struct ResponseChildProcessContext {
    pub response_process_num: usize,
    pub request_channel: Arc<RequestChannel>,
    pub connection_manager: Arc<ConnectionManager>,
    pub cache_manager: Arc<CacheManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub client_pool: Arc<ClientPool>,
    pub network_type: NetworkConnectionType,
    pub stop_sx: broadcast::Sender<bool>,
}

pub(crate) async fn response_process(mut context: ResponseProcessContext) {
    let mut stop_rx = context.stop_sx.subscribe();

    let child_context = ResponseChildProcessContext {
        response_process_num: context.response_process_num,
        request_channel: context.request_channel.clone(),
        connection_manager: context.connection_manager.clone(),
        cache_manager: context.cache_manager.clone(),
        subscribe_manager: context.subscribe_manager.clone(),
        client_pool: context.client_pool.clone(),
        network_type: context.network_type.clone(),
        stop_sx: context.stop_sx.clone(),
    };

    response_child_process(child_context);

    let mut response_process_seq = 0;
    tokio::spawn(async move {
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

                val = context.response_queue_rx.recv()=>{
                    if let Some(packet) = val{
                        let mut sleep_ms = 0;
                        metrics_response_queue_size("total", context.response_queue_rx.len());

                        loop {
                            response_process_seq += 1;
                            if let Some(handler_sx) = context.request_channel.get_available_response(&context.network_type,response_process_seq){
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

pub(crate) fn response_child_process(context: ResponseChildProcessContext) {
    for index in 1..=context.response_process_num {
        let mut response_process_rx = context
            .request_channel
            .create_response_child_channel(&context.network_type, index);
        let mut raw_stop_rx = context.stop_sx.subscribe();
        let raw_connect_manager = context.connection_manager.clone();
        let raw_cache_manager = context.cache_manager.clone();
        let raw_client_pool = context.client_pool.clone();
        let raw_subscribe_manager = context.subscribe_manager.clone();
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
