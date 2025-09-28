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
use crate::common::tool::read_packet;
use broker_core::cache::BrokerCacheManager;
use common_metrics::mqtt::packets::record_received_error_metrics;
use futures_util::StreamExt;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
use protocol::robust::RobustMQPacket;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver};
use tokio::time::sleep;
use tokio::{io, select};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, error};

/// The `acceptor_process` function is responsible for accepting incoming TCP connections
/// in an asynchronous manner. It utilizes multiple threads to handle the incoming connections
/// concurrently, improving the server's ability to manage a high volume of connections.
///
/// # Parameters
/// - `accept_thread_num`: The number of threads to spawn for accepting connections.
/// - `connection_manager`: An `Arc`-wrapped `ConnectionManager` instance for managing all network connections.
/// - `stop_sx`: A `broadcast::Sender` used to send stop signals to all acceptor threads.
/// - `listener_arc`: An `Arc`-wrapped `TcpListener` for listening to and accepting TCP connections.
/// - `request_queue_sx`: A `Sender` for sending `RequestPackage` instances to a processing queue.
/// - `cache_manager`: An `Arc`-wrapped `CacheManager` for managing cache operations.
/// - `network_connection_type`: An enum indicating the type of network connection.
///
#[allow(clippy::too_many_arguments)]
pub async fn acceptor_process(
    accept_thread_num: usize,
    connection_manager: Arc<ConnectionManager>,
    broker_cache: Arc<BrokerCacheManager>,
    stop_sx: broadcast::Sender<bool>,
    listener_arc: Arc<TcpListener>,
    request_channel: Arc<RequestChannel>,
    network_type: NetworkConnectionType,
    codec: RobustMQCodec,
) {
    for index in 1..=accept_thread_num {
        let listener = listener_arc.clone();
        let connection_manager = connection_manager.clone();
        let mut stop_rx = stop_sx.subscribe();
        let request_channel = request_channel.clone();
        let network_type = network_type.clone();
        let row_codec = codec.clone();
        let row_broker_cache = broker_cache.clone();
        tokio::spawn(async move {
            debug!(
                "{} Server acceptor thread {} start successfully.",
                network_type, index
            );
            loop {
                select! {
                    val = stop_rx.recv() => {
                        if let Ok(flag) = val {
                            if flag {
                                debug!("{} Server acceptor thread {} stopped successfully.", network_type, index);
                                break;
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

                                // if !tcp_establish_connection_check(&addr, &connection_manager, &mut write_frame_stream).await{
                                //     continue;
                                // }

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
    broker_cache: Arc<BrokerCacheManager>,
    mut read_frame_stream: FramedRead<io::ReadHalf<tokio::net::TcpStream>, RobustMQCodec>,
    connection_id: u64,
    connection_manager: Arc<ConnectionManager>,
    request_channel: Arc<RequestChannel>,
    mut connection_stop_rx: Receiver<bool>,
    network_type: NetworkConnectionType,
) {
    tokio::spawn(async move {
        loop {
            select! {
                val = connection_stop_rx.recv() =>{
                    if let Some(flag) = val{
                        if flag {
                            debug!("{} connection 【{}】 acceptor thread stopped successfully.", network_type, connection_id);
                            break;
                        }
                    }
                }

                package = read_frame_stream.next()=>{
                     if let Some(pkg) = package {
                        match pkg {
                            Ok(pack) => {
                                if broker_cache.is_stop(){
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
                                        read_packet(RobustMQPacket::KAFKA(pk.packet), &request_channel, &connection, &network_type).await;
                                    }
                                }

                            }
                            Err(e) => {
                                record_received_error_metrics(network_type.clone());
                                debug!(
                                    "{} connection parsing packet format error message :{:?}",
                                    network_type, e
                                )
                            }
                        }
                     }else{
                        sleep(Duration::from_millis(100)).await;
                     }
                }
            }
        }
    });
}
