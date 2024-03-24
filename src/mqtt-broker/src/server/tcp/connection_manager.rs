use super::connection::Connection;
use common_base::log::error_engine;
use dashmap::DashMap;
use protocol::mqtt::Packet;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Description The number of TCP connections on a node exceeded the upper limit. The maximum number of TCP connections was {total:?}")]
    ConnectionExceed { total: usize },
}

pub struct ConnectionManager {
    connections: DashMap<u64, Box<dyn Connection>>,
    max_connection_num: usize,
    max_try_mut_times: u64,
    try_mut_sleep_time_ms: u64,
}

impl ConnectionManager {
    pub fn new(
        max_connection_num: usize,
        max_try_mut_times: u64,
        try_mut_sleep_time_ms: u64,
    ) -> ConnectionManager {
        let connections = DashMap::with_capacity_and_shard_amount(1000, 64);
        ConnectionManager {
            connections,
            max_connection_num,
            max_try_mut_times,
            try_mut_sleep_time_ms,
        }
    }

    pub fn add(&self, connection: impl Connection) -> u64 {
        let connection_id = connection.connection_id();
        self.connections.insert(connection_id, Box::new(connection));
        return connection_id;
    }

    pub fn remove(&self, connection_id: u64) {
        self.connections.remove(&connection_id);
    }

    pub async fn write_frame(&self, connection_id: u64, resp: Packet) {
        let mut times = 0;
        loop {
            match self.connections.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    return da.write_frame(resp).await;
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > self.max_try_mut_times {
                        error_engine(format!("[write_frame]Connection management could not obtain an available connection. Connection ID: {}",connection_id));
                        break;
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > self.max_try_mut_times {
                        error_engine(format!("[write_frame]Connection management failed to get connection variable reference, connection ID: {}",connection_id));
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
