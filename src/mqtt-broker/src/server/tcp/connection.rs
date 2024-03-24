use axum::async_trait;
use common_base::log::error_meta;
use futures::SinkExt;
use protocol::{mqtt::Packet, mqttv4::codec::Mqtt4Codec, mqttv5::codec::Mqtt5Codec};
use std::{net::SocketAddr, sync::atomic::AtomicU64};
use tokio_util::codec::FramedWrite;

static CONNECTION_ID_BUILD: AtomicU64 = AtomicU64::new(1);

#[async_trait]
pub trait Connection {
    async fn write_frame(&mut self, resp: Packet);
    fn connection_id(&self) -> u64;
}
pub struct Mqtt4Connection {
    pub connection_id: u64,
    pub addr: SocketAddr,
    pub write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt4Codec>,
}

impl Mqtt4Connection {
    pub fn new(
        addr: SocketAddr,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt4Codec>,
    ) -> Mqtt4Connection {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Mqtt4Connection {
            connection_id,
            addr,
            write,
        }
    }
}

#[async_trait]
impl Connection for Mqtt4Connection {
    async fn write_frame(&mut self, resp: Packet) {
        match self.write.send(resp).await {
            Ok(_) => {}
            Err(err) => error_meta(&format!(
                "Failed to write data to the response queue, error message ff: {:?}",
                err
            )),
        }
    }
    fn connection_id(&self) -> u64 {
        return self.connection_id;
    }
}

pub struct Mqtt5Connection {
    pub connection_id: u64,
    pub addr: SocketAddr,
    pub write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>,
}

impl Mqtt5Connection {
    pub fn new(
        addr: SocketAddr,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>,
    ) -> Mqtt5Connection {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Mqtt5Connection {
            connection_id,
            addr,
            write,
        }
    }
}

#[async_trait]
impl Connection for Mqtt5Connection {
    async fn write_frame(&mut self, resp: Packet) {
        match self.write.send(resp).await {
            Ok(_) => {}
            Err(err) => error_meta(&format!(
                "Failed to write data to the response queue, error message ff: {:?}",
                err
            )),
        }
    }

    fn connection_id(&self) -> u64 {
        return self.connection_id;
    }
}
