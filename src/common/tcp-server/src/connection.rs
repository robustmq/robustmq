use common_base::log::error_meta;
use std::{net::SocketAddr, sync::atomic::AtomicU64};
use tokio_util::codec::{Decoder, Encoder, FramedWrite};

static CONNECTION_ID_BUILD: AtomicU64 = AtomicU64::new(1);

pub struct Connection<T, U> {
    pub connection_id: u64,
    pub addr: SocketAddr,
    pub write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, T>,
    pub packet: U,
}

impl<T, U> Connection<T, U> {
    pub fn new(
        addr: SocketAddr,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, T>,
        packet: U,
    ) -> Connection<T, U> {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Connection {
            connection_id,
            addr,
            write,
            packet,
        }
    }

    pub fn connection_id(&self) -> u64 {
        return self.connection_id;
    }

    pub async fn write_frame(&mut self, resp: U) {
        match self.write.send(resp).await {
            Ok(_) => {}
            Err(err) => error_meta(&format!(
                "Failed to write data to the response queue, error message ff: {:?}",
                err
            )),
        }
    }
}
