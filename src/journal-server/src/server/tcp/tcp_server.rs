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

use super::{
    connection::{Connection, ConnectionManager},
    packet::ResponsePackage,
};
use crate::{network::command::Command, server::tcp::packet::RequestPackage};

use futures::StreamExt;
use log::error;
use protocol::journal_server::codec::StorageEngineCodec;
use std::{fmt::Error, sync::Arc};
use tokio::{io, sync::broadcast};
use tokio::{net::TcpListener, sync::broadcast::Sender};
use tokio_util::codec::{FramedRead, FramedWrite};

pub struct TcpServer {
    connection_manager: Arc<ConnectionManager>,
    accept_thread_num: usize,
    handler_process_num: usize,
    response_process_num: usize,
    request_queue_sx: Sender<RequestPackage>,
    response_queue_sx: Sender<ResponsePackage>,
    codec: StorageEngineCodec,
}

impl TcpServer {
    pub fn new(
        accept_thread_num: usize,
        max_connection_num: usize,
        request_queue_size: usize,
        handler_process_num: usize,
        response_queue_size: usize,
        response_process_num: usize,
        max_try_mut_times: u64,
        try_mut_sleep_time_ms: u64,
    ) -> Self {
        let (request_queue_sx, _) = broadcast::channel(request_queue_size);
        let (response_queue_sx, _) = broadcast::channel(response_queue_size);

        let connection_manager = Arc::new(ConnectionManager::new(
            max_connection_num,
            max_try_mut_times,
            try_mut_sleep_time_ms,
        ));
        let codec = StorageEngineCodec::new();

        Self {
            connection_manager,
            accept_thread_num,
            handler_process_num,
            response_process_num,
            request_queue_sx,
            response_queue_sx,
            codec,
        }
    }

    pub async fn start(&self, port: u32) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .unwrap();
        let arc_listener = Arc::new(listener);

        for _ in 0..=self.accept_thread_num {
            _ = self.acceptor(arc_listener.clone()).await;
        }

        for _ in 0..=self.handler_process_num {
            _ = self.handler_process().await;
        }

        for _ in 0..=self.response_process_num {
            _ = self.response_process().await;
        }
    }

    async fn acceptor(&self, listener: Arc<TcpListener>) -> Result<(), Error> {
        let request_queue_sx = self.request_queue_sx.clone();
        let connection_manager = self.connection_manager.clone();
        let codec = self.codec.clone();
        tokio::spawn(async move {
            loop {
                let cm = connection_manager.clone();
                let request_queue_sx = request_queue_sx.clone();
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        match cm.connect_check() {
                            Ok(_) => {}
                            Err(e) => {
                                error!("tcp connection failed to establish from IP: {}. Failure reason: {}",addr.to_string(),e.to_string());
                                continue;
                            }
                        }

                        // split stream
                        let (r_stream, w_stream) = io::split(stream);
                        let mut read_frame_stream = FramedRead::new(r_stream, codec.clone());
                        let write_frame_stream = FramedWrite::new(w_stream, codec.clone());

                        // connection manager
                        let connection_id = cm.add(Connection::new(addr, write_frame_stream));

                        // request is processed by a separate thread, placing the request packet in the request queue.
                        tokio::spawn(async move {
                            loop {
                                if let Some(pkg) = read_frame_stream.next().await {
                                    match pkg {
                                        Ok(data) => {
                                            let package = RequestPackage::new(connection_id, data);
                                            match request_queue_sx.send(package) {
                                                Ok(_) => {}
                                                Err(err) => error!("Failed to write data to the request queue, error message: {:?}",err),
                                            }
                                        }
                                        Err(e) => {
                                            error!("{}", e);
                                        }
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                };
            }
        });

        return Ok(());
    }

    async fn handler_process(&self) -> Result<(), Error> {
        let mut request_queue_rx = self.request_queue_sx.subscribe();
        let response_queue_sx = self.response_queue_sx.clone();
        tokio::spawn(async move {
            while let Ok(resquest_package) = request_queue_rx.recv().await {
                //Business logic processing
                let command = Command::new(resquest_package.packet);
                let resp = command.apply();

                // Writes the result of the business logic processing to the return queue
                let response_package = ResponsePackage::new(resquest_package.connection_id, resp);
                match response_queue_sx.send(response_package) {
                    Ok(_) => {}
                    Err(err) => error!(
                        "Failed to write data to the response queue, error message: {:?}",
                        err
                    ),
                }
            }
        });
        return Ok(());
    }

    async fn response_process(&self) -> Result<(), Error> {
        let mut response_queue_rx = self.response_queue_sx.subscribe();
        let connect_manager = self.connection_manager.clone();
        tokio::spawn(async move {
            while let Ok(response_package) = response_queue_rx.recv().await {
                connect_manager
                    .write_frame(response_package.connection_id, response_package.packet)
                    .await;
            }
        });
        return Ok(());
    }
}
