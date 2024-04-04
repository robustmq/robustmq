use super::connection::Connection;
use common_base::log::{error, info};
use dashmap::DashMap;
use flume::r#async;
use futures::SinkExt;
use protocol::mqtt::MQTTPacket;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::codec::{Decoder, Encoder, FramedWrite};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Description The number of TCP connections on a node exceeded the upper limit. The maximum number of TCP connections was {total:?}")]
    ConnectionExceed { total: usize },
}

pub struct ConnectionManager<T> {
    connections: DashMap<u64, Connection>,
    write_list: DashMap<u64, FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, T>>,
    max_connection_num: usize,
    max_try_mut_times: u64,
    try_mut_sleep_time_ms: u64,
}

impl<T> ConnectionManager<T>
where
    T: Decoder + Encoder<MQTTPacket>,
{
    pub fn new(
        max_connection_num: usize,
        max_try_mut_times: u64,
        try_mut_sleep_time_ms: u64,
    ) -> ConnectionManager<T> {
        let connections = DashMap::with_capacity_and_shard_amount(1000, 64);
        let write_list = DashMap::with_capacity_and_shard_amount(1000, 64);
        ConnectionManager {
            connections,
            write_list,
            max_connection_num,
            max_try_mut_times,
            try_mut_sleep_time_ms,
        }
    }

    pub fn add(&self, connection: Connection) -> u64 {
        let connection_id = connection.connection_id();
        self.connections.insert(connection_id, connection);
        return connection_id;
    }

    pub fn add_write(
        &self,
        connection_id: u64,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, T>,
    ) {
        self.write_list.insert(connection_id, write);
    }

    pub async fn clonse_connect(&self, connection_id: u64) {
        self.connections.remove(&connection_id);
        if let Some((id, mut stream)) = self.write_list.remove(&connection_id) {
            match stream.close().await {
                Ok(_) => {
                    info(format!(
                        "server closes the connection actively, connection id [{}]",
                        id
                    ));
                }
                Err(_) => {}
            }
        }
    }

    pub async fn write_frame(&self, connection_id: u64, resp: MQTTPacket) {
        let mut times = 0;
        loop {
            match self.write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if times > self.max_try_mut_times {
                                error(format!(
                                    "Failed to write data to the response queue, error message: {:?}",
                                    "".to_string()
                                ));
                                break;
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > self.max_try_mut_times {
                        error(format!("[write_frame]Connection management could not obtain an available connection. Connection ID: {}",connection_id));
                        break;
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > self.max_try_mut_times {
                        error(format!("[write_frame]Connection management failed to get connection variable reference, connection ID: {}",connection_id));
                        break;
                    }
                }
            }
            times = times + 1;
            sleep(Duration::from_millis(self.try_mut_sleep_time_ms)).await
        }
    }

    pub fn connect_check(&self) -> Result<(), Error> {
        // Verify the connection limit
        if self.connections.len() >= self.max_connection_num {
            return Err(Error::ConnectionExceed {
                total: self.max_connection_num,
            });
        }

        // authentication
        return Ok(());
    }
}
