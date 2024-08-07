// Copyright 2023 RobustMQ Team
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


use crate::server::connection::NetworkConnection;
use crate::server::packet::RequestPackage;
use common_base::log::{debug, error, info};
use futures_util::StreamExt;
use protocol::mqtt::codec::MqttCodec;
use protocol::mqtt::common::MQTTPacket;
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::path::Path;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_util::codec::FramedRead;

pub(crate) fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

pub(crate) fn load_key(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    Ok(private_key(&mut BufReader::new(File::open(path)?))
        .unwrap()
        .ok_or(io::Error::new(
            ErrorKind::Other,
            "no private key found".to_string(),
        ))?)
}

pub(crate) fn read_tls_frame_process(
    mut read_frame_stream: FramedRead<
        tokio::io::ReadHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
        MqttCodec,
    >,
    connection: NetworkConnection,
    request_queue_sx: Sender<RequestPackage>,
    mut connection_stop_rx: Receiver<bool>,
) {
    tokio::spawn(async move {
        loop {
            select! {
                val = connection_stop_rx.recv() =>{
                    if let Some(flag) = val{
                        if flag {
                            debug(format!("TCP connection 【{}】 acceptor thread stopped successfully.",connection.connection_id));
                            break;
                        }
                    }
                }
                val = read_frame_stream.next()=>{
                    if let Some(pkg) = val {
                        match pkg {
                            Ok(data) => {
                                let pack: MQTTPacket = data.try_into().unwrap();
                                info(format!("revc tcp tls packet:{:?}", pack));
                                let package =
                                    RequestPackage::new(connection.connection_id, connection.addr, pack);
                                match request_queue_sx.send(package).await {
                                    Ok(_) => {
                                    }
                                    Err(err) => error(format!("Failed to write data to the request queue, error message: {:?}",err)),
                                }
                            }
                            Err(e) => {
                                debug(format!("TCP connection parsing packet format error message :{:?}",e))
                            }
                        }
                    }
                }
            }
        }
    });
}
