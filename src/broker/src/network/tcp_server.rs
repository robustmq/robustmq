use super::connection::{Connection, ConnectionManager};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use std::{fmt::Error, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub struct TcpServer {
    ip: SocketAddr,
    accept_thread_num: usize,
    max_connection_num: usize
}

impl TcpServer {
    pub fn new(ip: SocketAddr, accept_thread_num: usize, max_connection_num: usize) -> Self {
        Self {
            ip,
            accept_thread_num,
            max_connection_num,
        }
    }

    pub async fn start(&self) {
        let connection_manager =
            Arc::new(RwLock::new(ConnectionManager::new(self.max_connection_num)));
        _ = self.acceptor(connection_manager).await;
    }

    async fn acceptor(
        &self,
        connection_manager: Arc<RwLock<ConnectionManager>>,
    ) -> Result<(), Error> {
        let listener = TcpListener::bind(self.ip).await.unwrap();
        for _ in 0..=self.accept_thread_num {
            tokio::spawn(async move {
                let (stream, addr) = listener.accept().await.unwrap();
                tokio::spawn(async move {
                    let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
                    while let Some(Ok(data)) = stream.next().await {
                        println!("Got: {:?}", String::from_utf8_lossy(&data));

                        // 发送的消息也只需要发送消息主体，不需要提供长度
                        // Framed/LengthDelimitedCodec 会自动计算并添加
                        //    let response = &data[0..5];
                        stream.send(Bytes::from(data)).await.unwrap();
                    }
                });
                let conn = Connection::new(addr);
                let mut cm = connection_manager.write().await;
                _ = cm.add(conn);
            });
            return Ok(());
        }
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
        let net_s = TcpServer::new(ip, 10, 10);
    }
}
