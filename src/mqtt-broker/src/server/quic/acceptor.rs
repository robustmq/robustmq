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

use crate::observability::metrics::packets::record_received_error_metrics;
use crate::server::common::channel::RequestChannel;
use crate::server::common::connection::{NetworkConnection, NetworkConnectionType};
use crate::server::common::connection_manager::ConnectionManager;
use crate::server::common::tool::read_packet;
use crate::server::quic::stream::{QuicFramedReadStream, QuicFramedWriteStream};
use protocol::mqtt::codec::MqttCodec;
use quinn::Endpoint;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver};
use tracing::{debug, error, info};

pub(crate) async fn acceptor_process(
    accept_thread_num: usize,
    connection_manager: Arc<ConnectionManager>,
    endpoint_arc: Arc<Endpoint>,
    request_channel: Arc<RequestChannel>,
    network_type: NetworkConnectionType,
    stop_sx: broadcast::Sender<bool>,
) {
    for index in 1..=accept_thread_num {
        let endpoint = endpoint_arc.clone();
        let connection_manager = connection_manager.clone();
        let mut stop_rx = stop_sx.subscribe();
        let raw_request_channel = request_channel.clone();
        let network_type = network_type.clone();
        tokio::spawn(async move {
            debug!(
                "{} Server acceptor thread {} start successfully.",
                network_type, index
            );
            loop {
                select! {
                    val = stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                debug!("{} Server acceptor thread {} stopped successfully.", network_type, index);
                                break;
                            }
                        }
                    }
                    val = endpoint.accept()=> {
                        if let Some(incoming) = val{
                            match incoming.await {
                                Ok(connection) => {
                                    info!("Accept {} connection:{:?}", network_type, connection.remote_address());
                                    let client_addr = connection.remote_address();
                                    match connection.accept_bi().await {
                                        Ok((w_stream, r_stream)) => {
                                            let codec = MqttCodec::new(None);
                                            let codec_write = QuicFramedWriteStream::new(w_stream, codec.clone());
                                            let codec_read = QuicFramedReadStream::new(r_stream, codec.clone());
                                            // todo we need to add quic_establish_connection_check

                                            let (connection_stop_sx, connection_stop_rx) = mpsc::channel::<bool>(1);
                                            let connection = NetworkConnection::new(
                                                NetworkConnectionType::QUIC,
                                                client_addr,
                                                Some(connection_stop_sx.clone())
                                            );

                                            connection_manager.add_connection(connection.clone());
                                            connection_manager.add_quic_write(connection.connection_id, codec_write);

                                            read_frame_process(codec_read,  raw_request_channel.clone(),connection.clone(),network_type.clone(), connection_stop_rx)
                                        },
                                        Err(e) => {
                                            error!("{} accept failed to create connection with error message :{:?}", network_type, e);
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!("{} accept failed to wait connection with error message :{:?}", network_type, e);
                                }
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
    request_channel: Arc<RequestChannel>,
    connection: NetworkConnection,
    network_type: NetworkConnectionType,
    mut connection_stop_rx: Receiver<bool>,
) {
    tokio::spawn(async move {
        loop {
            select! {
                val = connection_stop_rx.recv() =>{
                    if let Some(flag) = val{
                        if flag {
                            debug!("{} connection 【{}】 acceptor thread stopped successfully.", network_type, connection.connection_id);
                            break;
                        }
                    }
                }
                package = read_frame_stream.receive() => {
                    match package {
                        Ok(pack) => {
                            read_packet(pack, &request_channel, &connection, &network_type).await;
                        }
                        Err(e) => {
                            record_received_error_metrics(network_type.clone());
                            debug!(
                                "{} connection parsing packet format error message :{:?}",
                                network_type, e
                            )
                        }
                    }
                }
            }
        }
    });
}
