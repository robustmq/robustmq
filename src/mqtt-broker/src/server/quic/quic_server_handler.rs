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
use crate::server::connection::NetworkConnectionType;
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::RequestPackage;
use log::{debug, error, info};
use protocol::mqtt::codec::MqttCodec;
use quinn::Endpoint;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
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
        let _connection_manager = connection_manager.clone();
        let mut stop_rx = stop_sx.subscribe();
        let _raw_request_queue_sx = request_queue_sx.clone();
        let _network_type = network_connection_type.clone();
        let _cache_manager = cache_manager.clone();
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
                                        match connection.accept_bi().await {
                                            Ok((_w_stream, _r_stream)) => {
                                                    let _codec = MqttCodec::new(None);
                                                    // todo 这里需要有对应的写入流
                                                   // todo 这里需要有对应的写出流

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
