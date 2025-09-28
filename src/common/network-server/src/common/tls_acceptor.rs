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

use broker_core::cache::BrokerCacheManager;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_config::broker::broker_config;
use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
use protocol::robust::RobustMQPacket;
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
use common_metrics::mqtt::packets::record_received_error_metrics;
use futures_util::StreamExt;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{broadcast, mpsc};
use tokio::time::sleep;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, error};

pub(crate) fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

pub(crate) fn load_key(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    private_key(&mut BufReader::new(File::open(path)?))
        .unwrap()
        .ok_or(io::Error::other("no private key found".to_string()))
}

#[allow(clippy::too_many_arguments)]
pub async fn acceptor_tls_process(
    accept_thread_num: usize,
    listener_arc: Arc<TcpListener>,
    stop_sx: broadcast::Sender<bool>,
    network_type: NetworkConnectionType,
    connection_manager: Arc<ConnectionManager>,
    broker_cache: Arc<BrokerCacheManager>,
    request_channel: Arc<RequestChannel>,
    codec: RobustMQCodec,
) -> ResultCommonError {
    let tls_acceptor = create_tls_accept()?;

    for index in 1..=accept_thread_num {
        let listener = listener_arc.clone();
        let connection_manager = connection_manager.clone();
        let mut stop_rx = stop_sx.subscribe();
        let request_channel = request_channel.clone();
        let raw_tls_acceptor = tls_acceptor.clone();
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
                    val = listener.accept()=>{
                        match val{
                            Ok((stream, addr)) => {
                                debug!("Accept {} tls connection:{:?}", network_type, addr);
                                let stream = match raw_tls_acceptor.accept(stream).await{
                                    Ok(da) => da,
                                    Err(e) => {
                                        error!("{} Accepter failed to read Stream with error message :{e:?}", network_type);
                                        continue;
                                    }
                                };

                                let (r_stream, w_stream) = tokio::io::split(stream);
                                let read_frame_stream = FramedRead::new(r_stream, row_codec.clone());
                                let write_frame_stream = FramedWrite::new(w_stream, row_codec.clone());

                                // if !tcp_tls_establish_connection_check(&addr,&connection_manager,&mut write_frame_stream).await{
                                //     continue;
                                // }

                                let (connection_stop_sx, connection_stop_rx) = mpsc::channel::<bool>(1);
                                let connection = NetworkConnection::new(
                                    NetworkConnectionType::Tls,
                                    addr,
                                    Some(connection_stop_sx.clone())
                                );
                                connection_manager.add_connection(connection.clone());
                                connection_manager.add_tcp_tls_write(connection.connection_id, write_frame_stream);

                                read_tls_frame_process(row_broker_cache.clone(), read_frame_stream, connection, request_channel.clone(), connection_stop_rx, network_type.clone());
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
    Ok(())
}

// spawn connection read thread
pub(crate) fn read_tls_frame_process(
    broker_cache: Arc<BrokerCacheManager>,
    mut read_frame_stream: FramedRead<
        tokio::io::ReadHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
        RobustMQCodec,
    >,
    connection: NetworkConnection,
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
                            debug!("{} connection 【{}】 acceptor thread stopped successfully.",network_type, connection.connection_id);
                            break;
                        }
                    }
                }
                package = read_frame_stream.next()=>{
                    if let Some(pkg) = package {
                        match pkg {
                            Ok(pack) => {
                                if broker_cache.is_stop(){
                                    debug!("{} connection 【{}】 acceptor thread stopped successfully.",network_type, connection.connection_id);
                                    break;
                                }
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

#[allow(clippy::result_large_err)]
fn create_tls_accept() -> Result<TlsAcceptor, CommonError> {
    let conf = broker_config();
    let certs = load_certs(Path::new(&conf.runtime.tls_cert))?;
    let key = load_key(Path::new(&conf.runtime.tls_key))?;
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}
