use crate::network::package::RequestPackage;

use super::{
    connection::{Connection, ConnectionManager},
    package::ResponsePackage,
};
use bytes::Bytes;
use common_log::log::error;
use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use std::{fmt::Error, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub struct TcpServer {
    ip: SocketAddr,
    accept_thread_num: usize,
    max_connection_num: usize,
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

        Self {
            ip,
            accept_thread_num,
            max_connection_num,
            handler_process_num,
            response_process_num,
            request_queue_sx,
            request_queue_rx,
            response_queue_sx,
            response_queue_rx,
        }
    }

    pub async fn start(&self) {
        let connection_manager =
            Arc::new(RwLock::new(ConnectionManager::new(self.max_connection_num)));

        let listener = TcpListener::bind(self.ip).await.unwrap();
        let arc_listener = Arc::new(listener);
        
        for _ in 0..=self.accept_thread_num {
            _ = self.acceptor(connection_manager.clone(),arc_listener.clone()).await;
        }

        for _ in 0..=self.handler_process_num {
            _ = self.handler_process().await;
        }

        for _ in 0..=self.response_process_num {
            _ = self.response_process().await;
        }
    }

    async fn acceptor(
        &self,
        connection_manager: Arc<RwLock<ConnectionManager>>,
        listener: Arc<TcpListener>,
    ) -> Result<(), Error> {
        let request_queue_sx = self.request_queue_sx.clone();
        tokio::spawn(async move {
            let (stream, addr) = listener.accept().await.unwrap();

            let conn = Connection::new(addr);
            let mut cm = connection_manager.write().await;
            _ = cm.add(conn.clone());

            tokio::spawn(async move {
                // todo Frames Codes need to be adjusted
                let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
                // let socket = Arc::new(RwLock::new(stream));
                // let r = socket.clone();

                // todo
                while let Some(Ok(data)) = stream.next().await {
                    let content = String::from_utf8_lossy(&data).to_string();
                    let package = RequestPackage::new(content, conn);
                    match request_queue_sx.send(package) {
                        Ok(_) => {}
                        Err(err) => error(&format!(
                            "Failed to write data to the request queue, error message: {:?}",
                            err
                        )),
                    }
                }
            });
        });

        return Ok(());
    }

    async fn handler_process(&self) -> Result<(), Error> {
        let request_queue_rx = self.request_queue_rx.clone();
        let response_queue_sx = self.response_queue_sx.clone();
        tokio::spawn(async move {
            while let Ok(resquest_package) = request_queue_rx.recv() {
                let resp = format!("robustmq response:{:?}", resquest_package);
                let response_package = ResponsePackage::new(resp);
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
        tokio::spawn(async move {
            while let Ok(response_package) = response_queue_rx.recv() {
                println!("{:?}", response_package);
            }
        });
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::TcpServer;
    use std::net::SocketAddr;

    #[test]
    fn network_server_test() {
        let ip: SocketAddr = "127.0.0.1:8768".parse().unwrap();
        let net_s = TcpServer::new(ip, 10, 10, 1, 1, 1, 1);
    }
}
