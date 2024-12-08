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

use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use log::{debug, error, info};
use protocol::mqtt::codec::MqttCodec;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::sleep;
use tokio::{io, select};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::handler::cache::CacheManager;
use crate::handler::validator::tcp_establish_connection_check;
use crate::observability::metrics::packets::{
    record_received_error_metrics, record_received_metrics,
};
use crate::observability::slow::request::try_record_total_request_ms;
use crate::server::connection::{NetworkConnection, NetworkConnectionType};
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::RequestPackage;

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
pub(crate) async fn acceptor_process(
    accept_thread_num: usize,
    connection_manager: Arc<ConnectionManager>,
    stop_sx: broadcast::Sender<bool>,
    listener_arc: Arc<TcpListener>,
    request_queue_sx: Sender<RequestPackage>,
    cache_manager: Arc<CacheManager>,
    network_connection_type: NetworkConnectionType,
) {
    for index in 1..=accept_thread_num {
        let listener = listener_arc.clone();
        let connection_manager = connection_manager.clone();
        let mut stop_rx = stop_sx.subscribe();
        let raw_request_queue_sx = request_queue_sx.clone();
        let network_type = network_connection_type.clone();
        let cache_manager = cache_manager.clone();
        tokio::spawn(async move {
            debug!("TCP Server acceptor thread {} start successfully.", index);
            loop {
                select! {
                    val = stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                debug!("TCP Server acceptor thread {} stopped successfully.",index);
                                break;
                            }
                        }
                    }
                    val = listener.accept()=>{
                        match val{
                            Ok((stream, addr)) => {
                                info!("accept tcp connection:{:?}",addr);

                                let (r_stream, w_stream) = io::split(stream);
                                let codec = MqttCodec::new(None);
                                let read_frame_stream = FramedRead::new(r_stream, codec.clone());
                                let mut  write_frame_stream = FramedWrite::new(w_stream, codec.clone());

                                if !tcp_establish_connection_check(&addr,&connection_manager,&mut write_frame_stream).await{
                                    continue;
                                }

                                let (connection_stop_sx, connection_stop_rx) = mpsc::channel::<bool>(1);
                                let connection = NetworkConnection::new(
                                    crate::server::connection::NetworkConnectionType::Tcp,
                                    addr,
                                    Some(connection_stop_sx.clone())
                                );
                                connection_manager.add_connection(connection.clone());
                                connection_manager.add_tcp_write(connection.connection_id, write_frame_stream);

                                read_frame_process(read_frame_stream,connection,raw_request_queue_sx.clone(),connection_stop_rx,network_type.clone(),cache_manager.clone());
                            }
                            Err(e) => {
                                error!("TCP accept failed to create connection with error message :{:?}",e);
                            }
                        }
                    }
                };
            }
        });
    }
}

fn read_frame_process(
    mut read_frame_stream: FramedRead<tokio::io::ReadHalf<tokio::net::TcpStream>, MqttCodec>,
    connection: NetworkConnection,
    request_queue_sx: Sender<RequestPackage>,
    mut connection_stop_rx: Receiver<bool>,
    network_type: NetworkConnectionType,
    cache_manager: Arc<CacheManager>,
) {
    tokio::spawn(async move {
        loop {
            select! {
                val = connection_stop_rx.recv() =>{
                    if let Some(flag) = val{
                        if flag {
                            debug!("TCP connection 【{}】 acceptor thread stopped successfully.",connection.connection_id);
                            break;
                        }
                    }
                }
                val = read_frame_stream.next()=>{
                    if let Some(pkg) = val {
                        match pkg {
                            Ok(pack) => {
                                record_received_metrics(&connection, &pack, &network_type);

                                info!("revc tcp packet:{:?}", pack);
                                let package =
                                    RequestPackage::new(connection.connection_id, connection.addr, pack);

                                match request_queue_sx.send(package.clone()).await {
                                    Ok(_) => {
                                        try_record_total_request_ms(cache_manager.clone(),package.clone());
                                    }
                                    Err(err) => error!("Failed to write data to the request queue, error message: {:?}",err),
                                }
                            }
                            Err(e) => {
                                record_received_error_metrics(network_type.clone());
                                debug!("TCP connection parsing packet format error message :{:?}",e)
                            }
                        }
                    }else {
                        sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        }
    });
}
