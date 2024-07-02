use super::connection::TCPConnection;
use crate::metrics::metrics_connection_num;
use common_base::log::{error, info};
use dashmap::DashMap;
use futures::SinkExt;
use protocol::mqtt::{
    codec::{MQTTPacketWrapper, MqttCodec},
    common::MQTTProtocol,
};
use std::{fmt::Debug, time::Duration};
use tokio::time::sleep;
use tokio_util::codec::FramedWrite;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Description The number of TCP connections on a node exceeded the upper limit. The maximum number of TCP connections was {total:?}")]
    ConnectionExceed { total: usize },
}

pub struct ConnectionManager {
    protocol: MQTTProtocol,
    connections: DashMap<u64, TCPConnection>,
    write_list: DashMap<u64, FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, MqttCodec>>,
    max_connection_num: usize,
    max_try_mut_times: u64,
    try_mut_sleep_time_ms: u64,
}

impl ConnectionManager {
    pub fn new(
        protocol: MQTTProtocol,
        max_connection_num: usize,
        max_try_mut_times: u64,
        try_mut_sleep_time_ms: u64,
    ) -> ConnectionManager {
        let connections = DashMap::with_capacity_and_shard_amount(1000, 64);
        let write_list = DashMap::with_capacity_and_shard_amount(1000, 64);
        ConnectionManager {
            protocol,
            connections,
            write_list,
            max_connection_num,
            max_try_mut_times,
            try_mut_sleep_time_ms,
        }
    }

    pub fn add(&self, connection: TCPConnection) -> u64 {
        let connection_id = connection.connection_id();
        self.connections.insert(connection_id, connection);
        let lable: String = self.protocol.clone().into();
        metrics_connection_num(&lable, self.connections.len() as i64);
        return connection_id;
    }

    pub fn add_write(
        &self,
        connection_id: u64,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, MqttCodec>,
    ) {
        self.write_list.insert(connection_id, write);
    }

    pub async fn clonse_connect(&self, connection_id: u64) {
        if let Some((_, connection)) = self.connections.remove(&connection_id) {
            connection.stop_connection();
        }

        if let Some((id, mut stream)) = self.write_list.remove(&connection_id) {
            // match stream.send().await {
            //     Ok(_) => {}
            //     Err(e) => {}
            // }
            match stream.close().await {
                Ok(_) => {
                    info(format!(
                        "server closes the connection actively, connection id [{}]",
                        id
                    ));
                }
                Err(e) => error(e.to_string()),
            }
        }
        let lable: String = self.protocol.clone().into();
        metrics_connection_num(&lable, self.connections.len() as i64);
    }

    pub async fn write_frame(&self, connection_id: u64, resp: MQTTPacketWrapper) {
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
                                    "Failed to write data to the mqtt client, error message: {:?}",
                                    e
                                ));
                                break;
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > self.max_try_mut_times {
                        error(format!("[write_frame]Connection management could not obtain an available connection. Connection ID: {},len:{}",connection_id,self.write_list.len()));
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

    pub fn connect_num_check(&self) -> bool {
        if self.connections.len() >= self.max_connection_num {
            return true;
        }
        return false;
    }

    pub fn connect_rate_check(&self) -> bool {
        return false;
    }

    pub fn get_connect(&self, connect_id: u64) -> Option<TCPConnection> {
        if let Some(connec) = self.connections.get(&connect_id) {
            return Some(connec.clone());
        }
        return None;
    }

    pub fn get_connect_protocol(&self, connect_id: u64) -> Option<MQTTProtocol> {
        if let Some(connec) = self.connections.get(&connect_id) {
            return connec.protocol.clone();
        }
        return None;
    }

    pub fn set_connect_protocol(&self, connect_id: u64, protocol: u8) {
        if let Some(mut connec) = self.connections.get_mut(&connect_id) {
            match protocol {
                3 => connec.set_protocol(MQTTProtocol::MQTT3),
                4 => connec.set_protocol(MQTTProtocol::MQTT4),
                5 => connec.set_protocol(MQTTProtocol::MQTT5),
                _ => {}
            };
        }
    }
}
