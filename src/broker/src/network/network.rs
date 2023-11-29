use std::{collections::VecDeque, io, time::Duration};

use bytes::BytesMut;
use protocol::{Packet, Protocol};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    time::error::Elapsed,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O = {0}")]
    Io(#[from] io::Error),
    #[error("Invalid data = {0}")]
    Protocol(#[from] protocol::Error),
    #[error["Keep alive timeout"]]
    KeepAlive(#[from] Elapsed),
}

pub struct Network<P> {
    socket: Box<dyn N>,
    read: BytesMut,
    write: BytesMut,
    max_incoming_size: usize,
    max_connection_buffer_len: usize,
    keepalive: Duration,
    protocol: P,
}

impl<P: Protocol> Network<P> {
    pub fn new(
        socket: Box<dyn N>,
        read_capacity: usize,
        write_capacity: usize,
        max_incoming_size: usize,
        max_connection_buffer_len: usize,
        protocol: P,
    ) -> Network<P> {
        Network {
            socket,
            read: BytesMut::with_capacity(read_capacity),
            write: BytesMut::with_capacity(write_capacity),
            max_incoming_size,
            max_connection_buffer_len,
            keepalive: Duration::from_secs(0),
            protocol,
        }
    }

    pub fn set_keepalive(&mut self, keepalive: u16) {
        self.keepalive = self.keepalive + Duration::from_secs(keepalive as u64).mul_f32(0.5)
    }

    async fn read_bytes(&mut self, required: usize) -> io::Result<usize> {
        let mut total_read = 0;
        loop {
            let read = self.socket.read_buf(&mut self.read).await?;
            if 0 == read {
                let error: io::Error = if self.read.is_empty() {
                    io::Error::new(
                        io::ErrorKind::ConnectionAborted,
                        "connection closed by peer",
                    )
                } else {
                    io::Error::new(io::ErrorKind::ConnectionReset, "connection reset by peer")
                };
                return Err(error);
            }

            total_read += read;
            if total_read >= required {
                return Ok(total_read);
            }
        }
    }

    pub async fn read(&mut self) -> Result<Packet, Error> {
        loop {
            let required =
                match Protocol::read(&mut self.protocol, &mut self.read, self.max_incoming_size) {
                    Ok(packet) => return Ok(packet),
                    Err(e) => return Err(e.into()),
                };
            self.read_bytes(required).await?;
        }
    }

    pub async fn write(&mut self, packet: Packet) -> Result<(), Error> {
        Protocol::write(&self.protocol, packet, &mut self.write)?;
        self.socket.write_all(&self.write).await?;
        self.write.clear();
        Ok(())
    }

    pub async fn write_bulk(&mut self, packets: VecDeque<Packet>) -> Result<(), Error> {
        for packet in packets {
            Protocol::write(&self.protocol, packet, &mut self.write)?;
        }
        self.socket.write_all(&self.write).await?;
        self.write.clear();
        Ok(())
    }
}

pub trait N: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T> N for T where T: AsyncRead + AsyncWrite + Send + Unpin {}
