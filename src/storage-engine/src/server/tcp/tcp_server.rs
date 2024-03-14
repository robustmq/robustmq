use crate::server::tcp::package::RequestPackage;
use bytes::BytesMut;
use tokio_util::codec::Framed;

use futures::{SinkExt, StreamExt};

use super::{
    connection::{Connection, ConnectionManager},
    package::ResponsePackage,
};
use common::{
    log::{error, info},
    metrics::broker::metrics_mqtt4_broker_running,
};
use flume::{Receiver, Sender};
use protocol::{
    mqtt::{ConnAck, ConnectReturnCode},
    mqttv4::codec::Mqtt4Codec,
};
use std::{fmt::Error, net::SocketAddr,sync::{Arc, RwLock}};
use tokio::net::TcpListener;

pub struct TcpServer {
    ip: SocketAddr,
    connection_manager: Arc<RwLock<ConnectionManager>>,
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
        ip: SocketAddr,
        accept_thread_num: usize,
        max_connection_num: usize,
        request_queue_size: usize,
        handler_process_num: usize,
        response_queue_size: usize,
        response_process_num: usize,
    ) -> Self {
        let (request_queue_sx, request_queue_rx) =
            flume::bounded::<RequestPackage>(request_queue_size);
        let (response_queue_sx, response_queue_rx) =
            flume::bounded::<ResponsePackage>(response_queue_size);

        let connection_manager: Arc<RwLock<ConnectionManager>> =
            Arc::new(RwLock::new(ConnectionManager::new(max_connection_num)));

        Self {
            ip,
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
        let listener = TcpListener::bind(self.ip).await.unwrap();
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
        metrics_mqtt4_broker_running();
        info(&format!(
            "RobustMQ Broker MQTT4 Server start success. bind addr:{:?}",
            self.ip
        ));
    }

    async fn acceptor(&self, listener: Arc<TcpListener>) -> Result<(), Error> {
        let request_queue_sx = self.request_queue_sx.clone();
        let connection_manager = self.connection_manager.clone();

        tokio::spawn(async move {
            loop {
                let request_queue_sx = request_queue_sx.clone();
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let mut socket: Framed<tokio::net::TcpStream, Mqtt4Codec> =
                            Framed::new(stream, Mqtt4Codec::new());

                        // manager connection info
                        let connection_manager = connection_manager.clone();
                        let conn = Connection::new(addr, socket.clone());
                        // let connection_id = conn.connection_id();
                        // let mut cm = connection_manager.write().await;
                        // _ = cm.add(conn);

                        let connection_id = 1;
                        // request is processed by a separate thread, placing the request packet in the request queue.\
                        tokio::spawn(async move {
                            while let Some(Ok(pkg)) = socket.next().await {
                                let content = format!("{:?}", pkg);
                                println!("receive package:{}", content);
                                let package = RequestPackage::new(100, content, connection_id);
                                match request_queue_sx.send(package) {
                                    Ok(_) => {}
                                    Err(err) => error(&format!(
                                "Failed to write data to the request queue, error message: {:?}",
                                err
                            )),
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
                let cm = connect_manager.write().await;
                let connection = cm.get(response_package.connection_id).unwrap();
                let mut stream = connection.socket.write().await;

                // send response
                let ack: ConnAck = ConnAck {
                    session_present: true,
                    code: ConnectReturnCode::Success,
                };

                match stream.send(&Packet::ConnAck(ack, None)).await {
                    Ok(_) => {}
                    Err(err) => error(&format!(
                        "Failed to write data to the response queue, error message ff: {:?}",
                        err
                    )),
                }

                println!("response data:{:?}", write_buf);
            }
        });
        return Ok(());
    }
}
