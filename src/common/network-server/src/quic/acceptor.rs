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
use crate::quic::stream::{QuicFramedReadStream, QuicFramedWriteStream};
use broker_core::cache::BrokerCacheManager;
use common_metrics::mqtt::packets::record_received_error_metrics;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
use protocol::robust::RobustMQPacket;
use quinn::{ConnectionError, Endpoint};
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver};
use tracing::{debug, error, info};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn acceptor_process(
    accept_thread_num: usize,
    connection_manager: Arc<ConnectionManager>,
    broker_cache: Arc<BrokerCacheManager>,
    endpoint_arc: Arc<Endpoint>,
    request_channel: Arc<RequestChannel>,
    network_type: NetworkConnectionType,
    codec: RobustMQCodec,
    stop_sx: broadcast::Sender<bool>,
) {
    for index in 1..=accept_thread_num {
        let endpoint = endpoint_arc.clone();
        let connection_manager = connection_manager.clone();
        let mut stop_rx = stop_sx.subscribe();
        let raw_request_channel = request_channel.clone();
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
                                            let codec_write = QuicFramedWriteStream::new(w_stream, row_codec.clone());
                                            let codec_read = QuicFramedReadStream::new(r_stream, row_codec.clone());

                                            // if !tcp_establish_connection_check(&addr, &connection_manager, &mut write_frame_stream).await{
                                            //     continue;
                                            // }

                                            let (connection_stop_sx, connection_stop_rx) = mpsc::channel::<bool>(1);
                                            let connection = NetworkConnection::new(
                                                NetworkConnectionType::QUIC,
                                                client_addr,
                                                Some(connection_stop_sx.clone())
                                            );

                                            connection_manager.add_connection(connection.clone());
                                            connection_manager.add_mqtt_quic_write(connection.connection_id, codec_write);

                                            read_frame_process(
                                                row_broker_cache.clone(),
                                                codec_read,
                                                connection.connection_id,
                                                connection_manager.clone(),
                                                raw_request_channel.clone(),
                                                connection_stop_rx,
                                                network_type.clone()
                                            );
                                        },
                                        Err(e) => {
                                            if let ConnectionError::ApplicationClosed(data) = e.clone(){
                                                if data.error_code.into_inner() == 0 {
                                                    continue;
                                                }
                                            }
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
    broker_cache: Arc<BrokerCacheManager>,
    mut read_frame_stream: QuicFramedReadStream,
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
                package = read_frame_stream.receive() => {
                    match package {
                        Ok(pack) => {
                             if broker_cache.is_stop(){
                                debug!("{} connection 【{}】 acceptor thread stopped successfully.", network_type, connection_id);
                                break;
                            }
                            if let Some(pk) = pack{
                                let connection = connection_manager.get_connect(connection_id).unwrap();
                                match pk {
                                    RobustMQCodecWrapper::MQTT(p) =>{
                                        read_packet(RobustMQPacket::MQTT(p.packet), &request_channel, &connection, &network_type).await;
                                    }
                                    RobustMQCodecWrapper::KAFKA(p) => {
                                        read_packet(RobustMQPacket::KAFKA(p.packet), &request_channel, &connection, &network_type).await;
                                    }
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
                }
            }
        }
    });
}
