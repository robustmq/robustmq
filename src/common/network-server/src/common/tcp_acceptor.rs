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

use crate::common::channel::RequestChannel;
use crate::common::connection_manager::ConnectionManager;
use crate::common::tool::{check_connection_limit, read_packet};
use broker_core::cache::NodeCacheManager;
use common_base::task::TaskSupervisor;
use common_metrics::mqtt::packets::record_received_error_metrics;
use futures_util::StreamExt;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
use protocol::robust::RobustMQPacket;
use rate_limit::global::GlobalRateLimiterManager;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver};
use tokio::{io, select};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, error, info, warn};

pub struct TcpAcceptorContext {
    pub accept_thread_num: usize,
    pub connection_manager: Arc<ConnectionManager>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub stop_sx: broadcast::Sender<bool>,
    pub listener: Arc<TcpListener>,
    pub request_channel: Arc<RequestChannel>,
    pub global_limit_manager: Arc<GlobalRateLimiterManager>,
    pub network_type: NetworkConnectionType,
    pub codec: RobustMQCodec,
    pub task_supervisor: Arc<TaskSupervisor>,
}

pub async fn acceptor_process(module_name: &str, ctx: TcpAcceptorContext) {
    for index in 1..=ctx.accept_thread_num {
        let listener = ctx.listener.clone();
        let connection_manager = ctx.connection_manager.clone();
        let mut stop_rx = ctx.stop_sx.subscribe();
        let request_channel = ctx.request_channel.clone();
        let network_type = ctx.network_type.clone();
        let row_codec = ctx.codec.clone();
        let row_broker_cache = ctx.broker_cache.clone();
        let row_global_limit_manager = ctx.global_limit_manager.clone();
        let task_name = format!("{}-{}-acceptor-{}", module_name, ctx.network_type, index);
        ctx.task_supervisor.spawn(task_name, async move {
            debug!(
                "{} Server acceptor thread {} start successfully.",
                network_type, index
            );
            loop {
                select! {
                    val = stop_rx.recv() => {
                        match val {
                            Ok(true) => {
                                debug!("{} Server acceptor thread {} stopped successfully.", network_type, index);
                                break;
                            }
                            Ok(false) => {}
                            Err(broadcast::error::RecvError::Closed) => {
                                debug!(
                                    "{} Server acceptor thread {} stop channel closed, exiting.",
                                    network_type, index
                                );
                                break;
                            }
                            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                                debug!(
                                    "{} Server acceptor thread {} lagged on stop channel, skipped {} messages.",
                                    network_type, index, skipped
                                );
                            }
                        }
                    }

                    val = listener.accept()=>{
                        match val{
                            Ok((stream, addr)) => {
                                debug!("Accept {} connection:{:?}", network_type, addr);

                                let (r_stream, w_stream) = io::split(stream);
                                let read_frame_stream = FramedRead::new(r_stream, row_codec.clone());
                                let write_frame_stream = FramedWrite::new(w_stream, row_codec.clone());

                                if let Err(e) = check_connection_limit(&row_global_limit_manager, &row_broker_cache, &connection_manager).await{
                                    warn!("{}",e.to_string());
                                    continue;
                                }

                                let (connection_stop_sx, connection_stop_rx) = mpsc::channel::<bool>(1);
                                let connection = NetworkConnection::new(
                                    NetworkConnectionType::Tcp,
                                    addr,
                                    Some(connection_stop_sx.clone())
                                );

                                connection_manager.add_connection(connection.clone());
                                connection_manager.add_tcp_write(connection.connection_id, write_frame_stream);
                                read_frame_process(
                                    row_broker_cache.clone(),
                                    read_frame_stream,
                                    connection.connection_id(),
                                    connection_manager.clone(),
                                    request_channel.clone(),
                                    connection_stop_rx,
                                    network_type.clone(),
                                );
                            }
                            Err(e) => {
                                error!("{} accept failed to create connection with error message :{:?}", network_type, e);
                            }
                        }
                    }
                };
            }
        });
    }
}

// spawn connection read thread
fn read_frame_process(
    broker_cache: Arc<NodeCacheManager>,
    mut read_frame_stream: FramedRead<io::ReadHalf<tokio::net::TcpStream>, RobustMQCodec>,
    connection_id: u64,
    connection_manager: Arc<ConnectionManager>,
    request_channel: Arc<RequestChannel>,
    mut connection_stop_rx: Receiver<bool>,
    network_type: NetworkConnectionType,
) {
    tokio::spawn(Box::pin(async move {
        loop {
            select! {
                val = connection_stop_rx.recv() =>{
                    match val {
                        Some(true) => {
                            debug!("{} connection 【{}】 acceptor thread stopped successfully.", network_type, connection_id);
                            break;
                        }
                        Some(false) => {}
                        None => {
                            debug!(
                                "{} connection 【{}】 stop channel closed, exiting read loop.",
                                network_type, connection_id
                            );
                            break;
                        }
                    }
                }

                package = read_frame_stream.next()=>{
                     if let Some(pkg) = package {
                        match pkg {
                            Ok(pack) => {
                                info!("recv package: {:?}",pack);
                                if broker_cache.is_stop().await{
                                    debug!("{} connection 【{}】 acceptor thread stopped successfully.", network_type, connection_id);
                                    break;
                                }
                                let connection = if let Some(conn) = connection_manager.get_connect(connection_id){
                                    conn
                                }else{
                                    continue;
                                };
                                match pack{
                                    RobustMQCodecWrapper::MQTT(pk) =>{
                                        read_packet(RobustMQPacket::MQTT(pk.packet), &request_channel, &connection, &network_type).await;
                                    }
                                    RobustMQCodecWrapper::KAFKA(pk) => {
                                        read_packet(RobustMQPacket::KAFKA(pk), &request_channel, &connection, &network_type).await;
                                    }
                                    RobustMQCodecWrapper::AMQP(pk) => {
                                        read_packet(RobustMQPacket::AMQP(pk), &request_channel, &connection, &network_type).await;
                                    }
                                    RobustMQCodecWrapper::StorageEngine(pk) => {
                                        read_packet(RobustMQPacket::StorageEngine(pk), &request_channel, &connection, &network_type).await;
                                    }
                                    RobustMQCodecWrapper::NATS(pkt) => {
                                        read_packet(RobustMQPacket::NATS(pkt), &request_channel, &connection, &network_type).await;
                                    }
                                }

                            }
                            Err(e) => {
                                record_received_error_metrics(network_type.clone());
                                debug!(
                                    "{} connection parsing packet format error message :{:?}",
                                    network_type, e
                                );
                                connection_manager.mark_close_connect(connection_id).await;
                                break;
                            }
                        }
                     }else{
                        debug!("Tcp client disconnected (EOF): connection_id={}", connection_id);
                        connection_manager.mark_close_connect(connection_id).await;
                        break;
                     }
                }
            }
        }
    }));
}
