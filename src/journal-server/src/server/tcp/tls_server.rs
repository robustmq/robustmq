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

use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use common_base::config::journal_server::journal_server_conf;
use futures_util::StreamExt;
use log::{debug, error, info};
use protocol::journal_server::codec::JournalServerCodec;
use rustls_pemfile::{certs, private_key};
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc};
use tokio::time::sleep;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::server::connection::{NetworkConnection, NetworkConnectionType};
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::RequestPackage;

pub(crate) fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

pub(crate) fn load_key(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    private_key(&mut BufReader::new(File::open(path)?))
        .unwrap()
        .ok_or(io::Error::new(
            ErrorKind::Other,
            "no private key found".to_string(),
        ))
}

pub(crate) async fn acceptor_tls_process(
    accept_thread_num: usize,
    listener_arc: Arc<TcpListener>,
    stop_sx: broadcast::Sender<bool>,
    network_connection_type: NetworkConnectionType,
    connection_manager: Arc<ConnectionManager>,
    request_queue_sx: Sender<RequestPackage>,
) {
    let conf = journal_server_conf();
    let certs = match load_certs(Path::new(&conf.network.tls_cert)) {
        Ok(data) => data,
        Err(e) => {
            panic!("{}", e.to_string());
        }
    };

    let key = match load_key(Path::new(&conf.network.tls_key)) {
        Ok(data) => data,
        Err(e) => {
            panic!("{}", e.to_string());
        }
    };

    let config = match ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
    {
        Ok(data) => data,
        Err(e) => {
            panic!("{}", e.to_string());
        }
    };
    let tls_acceptor = TlsAcceptor::from(Arc::new(config));

    for index in 1..=accept_thread_num {
        let listener = listener_arc.clone();
        let connection_manager = connection_manager.clone();
        let mut stop_rx = stop_sx.subscribe();
        let raw_request_queue_sx = request_queue_sx.clone();
        let raw_tls_acceptor = tls_acceptor.clone();
        let network_type = network_connection_type.clone();
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
                                info!("accept tcp tls connection:{:?}",addr);
                                let stream = match raw_tls_acceptor.accept(stream).await{
                                    Ok(da) => da,
                                    Err(e) => {
                                        error!("Tls Accepter failed to read Stream with error message :{e:?}");
                                        continue;
                                    }
                                };
                                let (r_stream, w_stream) = tokio::io::split(stream);
                                let codec = JournalServerCodec::new();
                                let read_frame_stream = FramedRead::new(r_stream, codec.clone());
                                let write_frame_stream = FramedWrite::new(w_stream, codec.clone());

                                // if !tcp_tls_establish_connection_check(&addr,&connection_manager,&mut write_frame_stream).await{
                                //     continue;
                                // }

                                let (connection_stop_sx, connection_stop_rx) = mpsc::channel::<bool>(1);
                                let connection = NetworkConnection::new(
                                    crate::server::connection::NetworkConnectionType::Tls,
                                    addr,
                                    Some(connection_stop_sx.clone())
                                );
                                connection_manager.add_connection(connection.clone());
                                connection_manager.add_tcp_tls_write(connection.connection_id, write_frame_stream);

                                read_tls_frame_process(read_frame_stream,connection,raw_request_queue_sx.clone(),connection_stop_rx, network_type.clone());
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

pub(crate) fn read_tls_frame_process(
    mut read_frame_stream: FramedRead<
        tokio::io::ReadHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
        JournalServerCodec,
    >,
    connection: NetworkConnection,
    request_queue_sx: Sender<RequestPackage>,
    mut connection_stop_rx: Receiver<bool>,
    network_type: NetworkConnectionType,
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

                                info!("revc tcp tls packet:{:?}", pack);
                                let package =
                                    RequestPackage::new(connection.connection_id, connection.addr, pack);
                                match request_queue_sx.send(package).await {
                                    Ok(_) => {
                                    }
                                    Err(err) => error!("Failed to write data to the request queue, error message: {:?}",err),
                                }
                            }
                            Err(e) => {
                                debug!("TCP connection parsing packet format error message :{:?}",e)
                            }
                        }
                    } else {
                        sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        }
    });
}
