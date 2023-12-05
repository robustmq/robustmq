use super::{
    connection::{Connection, ConnectionManager},
    network::Network,
    package::ResponsePackage,
};
use crate::{network::package::RequestPackage, package::mqtt4::package_ack_write};
use common::log::{info, error};
use flume::{Receiver, Sender};
use protocol::{mqttv4::MqttV4, protocol::ConnectReturnCode};
use std::{fmt::Error, net::SocketAddr, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    sync::RwLock,
};

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

        info(&format!("RobustMQ Broker TCP Server start success. bind addr:{:?}", self.ip));
    }

    async fn acceptor(&self, listener: Arc<TcpListener>) -> Result<(), Error> {
        let request_queue_sx = self.request_queue_sx.clone();
        let connection_manager = self.connection_manager.clone();

        tokio::spawn(async move {
            loop {
                let request_queue_sx = request_queue_sx.clone();
                let connection_manager = connection_manager.clone();
                let (stream, addr) = listener.accept().await.unwrap();
                let socket = Arc::new(RwLock::new(Box::new(stream)));
                let prot = MqttV4::new();
   
                // read connect package

                // tls check

                // user login check

                // manager connection info
                let conn = Connection::new(addr, socket.clone());
                let connection_id = conn.connection_id();
                let mut cm = connection_manager.write().await;
                _ = cm.add(conn);

                // request is processed by a separate thread, placing the request packet in the request queue.\
                tokio::spawn(async move {
                    let mut network = Network::new(socket.clone(), 2000, 2000, 2000, 3000, prot);
                    
                    match network.read().await {
                        Ok(pkg) => {
                            let content = format!("{:?}",pkg);
                            println!("receive package:{}",content);
                            let package = RequestPackage::new(100, content, connection_id);
                            match request_queue_sx.send(package) {
                                Ok(_) => {}
                                Err(err) => error(&format!(
                                "Failed to write data to the request queue, error message: {:?}",
                                err
                            )),
                            }
                        }
                        Err(err) => error(&format!(
                            "Failed to read data from TCP connection, error message: {:?}",
                            err
                        )),
                    }
                });
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
                let write_buf = package_ack_write(false,ConnectReturnCode::Success);
                match stream.write_all(&write_buf).await {
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
