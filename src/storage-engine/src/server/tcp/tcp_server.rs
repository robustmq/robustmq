use super::{
    connection::{Connection, ConnectionManager},
    package::ResponsePackage,
};
use crate::server::tcp::package::RequestPackage;
use common::log::error;
use flume::{Receiver, Sender};
use protocol::{
    mqtt::{ConnAck, ConnectReturnCode, Packet},
    mqttv4::codec::Mqtt4Codec,
};
use std::{fmt::Error, sync::Arc};
use tokio::net::TcpListener;
use tokio_util::codec::Framed;

pub struct TcpServer {
    connection_manager: Arc<ConnectionManager>,
    accept_thread_num: usize,
    handler_process_num: usize,
    response_process_num: usize,
    request_queue_sx: Sender<RequestPackage>,
    request_queue_rx: Receiver<RequestPackage>,
    response_queue_sx: Sender<ResponsePackage>,
    response_queue_rx: Receiver<ResponsePackage>,
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
        let (request_queue_sx, request_queue_rx) =
            flume::bounded::<RequestPackage>(request_queue_size);
        let (response_queue_sx, response_queue_rx) =
            flume::bounded::<ResponsePackage>(response_queue_size);

        let connection_manager = Arc::new(ConnectionManager::new(
            max_connection_num,
            max_try_mut_times,
            try_mut_sleep_time_ms,
        ));

        Self {
            connection_manager,
            accept_thread_num,
            handler_process_num,
            response_process_num,
            request_queue_sx,
            request_queue_rx,
            response_queue_sx,
            response_queue_rx,
        }
    }

    pub async fn start(&self) {
        let listener = TcpListener::bind("0.0.0.0").await.unwrap();
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

        tokio::spawn(async move {
            loop {
                let cm = connection_manager.clone();
                let request_queue_sx = request_queue_sx.clone();
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        match cm.connect_check() {
                            Ok(_) => {}
                            Err(e) => {
                                error(&format!("tcp connection failed to establish from IP: {}. Failure reason: {}",addr.to_string(),e.to_string()));
                                continue;
                            }
                        }

                        let stream = Framed::new(stream, Mqtt4Codec::new());
                        let connection_id = cm.add(Connection::new(addr, stream));

                        // request is processed by a separate thread, placing the request packet in the request queue.
                        tokio::spawn(async move {
                            while let Some(pkg) = cm.read_frame(connection_id).await {
                                let content = format!("{:?}", pkg);
                                let package = RequestPackage::new(100, content, connection_id);
                                match request_queue_sx.send(package) {
                                    Ok(_) => {}
                                    Err(err) => error(&format!("Failed to write data to the request queue, error message: {:?}",err)),
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error(&e.to_string());
                    }
                };
            }
        });

        return Ok(());
    }

    async fn handler_process(&self) -> Result<(), Error> {
        let request_queue_rx = self.request_queue_rx.clone();
        let response_queue_sx = self.response_queue_sx.clone();
        tokio::spawn(async move {
            while let Ok(resquest_package) = request_queue_rx.recv() {
                // todo Business logic processing

                // Writes the result of the business logic processing to the return queue
                let resp = format!("robustmq response:{:?}", resquest_package);
                let response_package = ResponsePackage::new(resp, resquest_package.connection_id);
                match response_queue_sx.send(response_package) {
                    Ok(_) => {}
                    Err(err) => error(&format!(
                        "Failed to write data to the response queue, error message: {:?}",
                        err
                    )),
                }
            }
        });
        return Ok(());
    }

    async fn response_process(&self) -> Result<(), Error> {
        let response_queue_rx = self.response_queue_rx.clone();
        let connect_manager = self.connection_manager.clone();
        tokio::spawn(async move {
            while let Ok(response_package) = response_queue_rx.recv() {
                // Logical processing of data response

                // Write the data back to the client
                let ack: ConnAck = ConnAck {
                    session_present: true,
                    code: ConnectReturnCode::Success,
                };
                let resp = Packet::ConnAck(ack, None);
                connect_manager
                    .write_frame(response_package.connection_id, resp)
                    .await;
            }
        });
        return Ok(());
    }
}
