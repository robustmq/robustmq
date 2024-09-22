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

use crate::{
    handler::{
        cache::CacheManager,
        command::Command,
        connection::disconnect_connection,
        validator::{tcp_establish_connection_check, tcp_tls_establish_connection_check},
    },
    observability::{
        metrics::{
            packets::{record_received_error_metrics, record_received_metrics},
            server::{metrics_request_queue, metrics_response_queue},
        },
        slow::request::try_record_total_request_ms,
    },
    server::{
        connection::{NetworkConnection, NetworkConnectionType},
        connection_manager::ConnectionManager,
        packet::{RequestPackage, ResponsePackage},
        tcp::{
            handler::handler_process,
            tls_server::{acceptor_tls_process, read_tls_frame_process},
        },
    },
    subscribe::subscribe_manager::SubscribeManager,
};
use clients::poll::ClientPool;
use common_base::{config::broker_mqtt::broker_mqtt_conf, error::mqtt_broker::MQTTBrokerError};
use futures_util::StreamExt;
use log::{debug, error, info};
use protocol::mqtt::{
    codec::{MQTTPacketWrapper, MqttCodec},
    common::MQTTPacket,
};
use std::{collections::HashMap, path::Path, sync::Arc};
use storage_adapter::storage::StorageAdapter;
use tokio::{
    io, select,
    sync::mpsc::{self, Receiver, Sender},
};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use tokio_util::codec::{FramedRead, FramedWrite};

use super::{
    response::response_child_process,
    tls_server::{load_certs, load_key},
};

// U: codec: encoder + decoder
// S: message storage adapter
pub struct TcpServer<S> {
    command: Command<S>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_poll: Arc<ClientPool>,
    accept_thread_num: usize,
    handler_process_num: usize,
    response_process_num: usize,
    stop_sx: broadcast::Sender<bool>,
    network_connection_type: NetworkConnectionType,
}

impl<S> TcpServer<S>
where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    pub fn new(
        command: Command<S>,
        accept_thread_num: usize,
        handler_process_num: usize,
        response_process_num: usize,
        stop_sx: broadcast::Sender<bool>,
        connection_manager: Arc<ConnectionManager>,
        subscribe_manager: Arc<SubscribeManager>,
        cache_manager: Arc<CacheManager>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        Self {
            command,
            subscribe_manager,
            cache_manager,
            client_poll,
            connection_manager,
            accept_thread_num,
            handler_process_num,
            response_process_num,
            stop_sx,
            network_connection_type: NetworkConnectionType::TCP,
        }
    }



    async fn acceptor_process(
        &self,
        listener_arc: Arc<TcpListener>,
        request_queue_sx: Sender<RequestPackage>,
    ) {
        for index in 1..=self.accept_thread_num {
            let listener = listener_arc.clone();
            let connection_manager = self.connection_manager.clone();
            let mut stop_rx = self.stop_sx.subscribe();
            let raw_request_queue_sx = request_queue_sx.clone();
            let network_type = self.network_connection_type.clone();
            let cache_manager = self.cache_manager.clone();
            tokio::spawn(async move {
                debug!("TCP Server acceptor thread {} start successfully.", index);
                loop {
                    select! {
                        val = stop_rx.recv() =>{
                            match val{
                                Ok(flag) => {
                                    if flag {
                                        debug!("TCP Server acceptor thread {} stopped successfully.",index);
                                        break;
                                    }
                                }
                                Err(_) => {}
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
                                        crate::server::connection::NetworkConnectionType::TCP,
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

    async fn response_process(&self, mut response_queue_rx: Receiver<ResponsePackage>) {
        let connect_manager = self.connection_manager.clone();
        let mut stop_rx = self.stop_sx.subscribe();
        let response_process_num = self.response_process_num.clone();
        let cache_manager = self.cache_manager.clone();
        let client_poll = self.client_poll.clone();
        let subscribe_manager = self.subscribe_manager.clone();
        let stop_sx = self.stop_sx.clone();

        tokio::spawn(async move {
            let mut process_handler: HashMap<usize, Sender<ResponsePackage>> = HashMap::new();
            response_child_process(
                response_process_num,
                &mut process_handler,
                stop_sx,
                connect_manager,
                cache_manager,
                subscribe_manager,
                client_poll,
            );

            let mut response_process_seq = 1;
            loop {
                select! {
                    val = stop_rx.recv() =>{
                        match val{
                            Ok(flag) => {
                                if flag {
                                    debug!("{}","TCP Server response process thread stopped successfully.");
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    }

                    val = response_queue_rx.recv()=>{
                        if let Some(packet) = val{
                            metrics_request_queue("response-total", response_queue_rx.len());
                            loop{
                                let seq = if response_process_seq > process_handler.len(){
                                    1
                                } else {
                                    response_process_seq
                                };

                                if let Some(handler_sx) = process_handler.get(&seq){
                                    match handler_sx.try_send(packet.clone()){
                                        Ok(_) => {
                                            break;
                                        }
                                        Err(err) => error!(
                                            "Failed to write data to the response process queue, error message: {:?}",
                                            err
                                        ),
                                    }
                                    response_process_seq = response_process_seq + 1;
                                }else{
                                    error!("{}","No request packet processing thread available");
                                }
                            }
                        }
                    }
                }
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
                            Ok(data) => {
                                let pack: MQTTPacket = data.try_into().unwrap();
                                record_received_metrics(&connection, &pack, &network_type);

                                debug!("revc tcp packet:{:?}", pack);
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

                    }
                }
            }
        }
    });
}
