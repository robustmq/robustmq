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
use crate::observability::metrics::packets::{
    record_received_error_metrics, record_received_metrics,
};
use crate::observability::slow::request::try_record_total_request_ms;
use crate::server::connection::{NetworkConnection, NetworkConnectionType};
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::RequestPackage;
use crate::server::quic::quic_stream_wrapper::{QuicFramedReadStream, QuicFramedWriteStream};
use log::{debug, error, info};
use protocol::mqtt::codec::MqttCodec;
use quinn::Endpoint;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver, Sender};

#[allow(dead_code)]
pub(crate) async fn acceptor_process(
    accept_thread_num: usize,
    connection_manager: Arc<ConnectionManager>,
    stop_sx: broadcast::Sender<bool>,
    endpoint_arc: Arc<Endpoint>,
    request_queue_sx: Sender<RequestPackage>,
    cache_manager: Arc<CacheManager>,
    network_connection_type: NetworkConnectionType,
) {
    for index in 1..=accept_thread_num {
        let endpoint = endpoint_arc.clone();
        let connection_manager = connection_manager.clone();
        let mut stop_rx = stop_sx.subscribe();
        let raw_request_queue_sx = request_queue_sx.clone();
        let network_type = network_connection_type.clone();
        let cache_manager = cache_manager.clone();
        tokio::spawn(async move {
            debug!("Quic Server acceptor thread {} start successfully.", index);
            loop {
                select! {
                    val = stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                debug!("Quic Server acceptor thread {} stopped successfully.",index);
                                break;
                            }
                        }
                    }
                    // todo 这部分需要修改
                    val = endpoint.accept()=> {
                        match val {
                            Some(incoming) => {
                                match incoming.await {
                                Ok(connection) => {
                                        info!("accept quic connection:{:?}",connection.remote_address());
                                        let client_addr = connection.remote_address();
                                        match connection.accept_bi().await {
                                            Ok((w_stream, r_stream)) => {
                                                    let codec = MqttCodec::new(None);
                                                    let quic_framed_write_stream = QuicFramedWriteStream::new(w_stream, codec.clone());
                                                    let quic_framed_read_stream = QuicFramedReadStream::new(r_stream, codec.clone());
                                                    // todo we need to add quic_establish_connection_check

                                                let (connection_stop_sx, connection_stop_rx) = mpsc::channel::<bool>(1);
                                                let connection = NetworkConnection::new(
                                                    NetworkConnectionType::Quic,
                                                    client_addr,
                                                    Some(connection_stop_sx.clone())
                                                );
                                                connection_manager.add_connection(connection.clone());
                                                connection_manager.add_quic_write(connection.connection_id, quic_framed_write_stream);
                                                read_frame_process(quic_framed_read_stream, connection.clone(), raw_request_queue_sx.clone(),connection_stop_rx, network_type.clone(), cache_manager.clone())
                                            },
                                            Err(e) => {
                                                error!("Quic accept failed to create connection with error message :{:?}",e);
                                            }
                                        }
                                },
                                Err(e) => {
                                        error!("Quic accept failed to create connection with error message :{:?}",e);
                                    }
                                }
                            },
                            None => {
                                error!("Quic Server acceptor thread {} stopped unexpectedly.",index);
                            }

                        }
                    }
                };
            }
        });
    }
}

fn read_frame_process(
    mut read_frame_stream: QuicFramedReadStream,
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
                val = read_frame_stream.receive() => {
                      match val {

                            Ok(packet) => {
                                    record_received_metrics(&connection, &packet, &network_type);

                                    info!("revc quic packet:{:?}", packet);
                                    let package =
                                        RequestPackage::new(connection.connection_id, connection.addr, packet);

                                    match request_queue_sx.send(package.clone()).await {
                                        Ok(_) => {
                                            try_record_total_request_ms(cache_manager.clone(),package.clone());
                                        }
                                        Err(err) => error!("Failed to write data to the request queue, error message: {:?}",err),
                                    }
                                },
                            Err(e) => {
                                record_received_error_metrics(network_type.clone());
                                debug!("Quic connection parsing packet format error message :{:?}",e)
                            }
                    }
                }
            }
        }
    });
}
