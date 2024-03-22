use common_base::log::error_engine;
use dashmap::DashMap;
use std::time::Duration;
use tokio::time::sleep;

use crate::{connection::Connection, error::Error};

pub struct ConnectionManager<T, U> {
    connections: DashMap<u64, Connection<T, U>>,
    max_connection_num: usize,
    max_try_mut_times: u64,
    try_mut_sleep_time_ms: u64,
}

impl<T, U> ConnectionManager<T, U> {
    pub fn new(
        max_connection_num: usize,
        max_try_mut_times: u64,
        try_mut_sleep_time_ms: u64,
    ) -> ConnectionManager<T, U> {
        let connections: DashMap<u64, Connection> =
            DashMap::with_capacity_and_shard_amount(1000, 64);
        ConnectionManager {
            connections,
            max_connection_num,
            max_try_mut_times,
            try_mut_sleep_time_ms,
        }
    }

    pub fn add(&self, connection: Connection<T, U>) -> u64 {
        let connection_id = connection.connection_id;
        self.connections.insert(connection_id, connection);
        return connection_id;
    }

    pub fn remove(&self, connection_id: u64) {
        self.connections.remove(&connection_id);
    }

    pub async fn write_frame(&self, connection_id: u64, resp: U) {
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
