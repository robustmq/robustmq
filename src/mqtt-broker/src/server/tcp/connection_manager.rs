use crate::handler::cache_manager::CacheManager;
use super::connection::NetworkConnection;
use common_base::log::{error, info};
use dashmap::DashMap;
use futures::SinkExt;
use protocol::mqtt::{
    codec::{MQTTPacketWrapper, MqttCodec},
    common::MQTTProtocol,
};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tokio_util::codec::FramedWrite;

pub struct ConnectionManager {
    connections: DashMap<u64, NetworkConnection>,
    write_list: DashMap<u64, FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, MqttCodec>>,
    cache_manager: Arc<CacheManager>,
}

impl ConnectionManager {
    pub fn new(cache_manager: Arc<CacheManager>) -> ConnectionManager {
        let connections = DashMap::with_capacity_and_shard_amount(1000, 64);
        let write_list = DashMap::with_capacity_and_shard_amount(1000, 64);
        ConnectionManager {
            connections,
            write_list,
            cache_manager,
        }
    }

    pub fn add(&self, connection: NetworkConnection) -> u64 {
        let connection_id = connection.connection_id();
        self.connections.insert(connection_id, connection);
        return connection_id;
    }

    pub fn add_write(
        &self,
        connection_id: u64,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, MqttCodec>,
    ) {
        self.write_list.insert(connection_id, write);
    }

    pub async fn close_all_connect(&self) {
        for (connect_id, _) in self.connections.clone() {
            self.clonse_connect(connect_id).await;
        }
    }

    pub async fn clonse_connect(&self, connection_id: u64) {
        if let Some((_, connection)) = self.connections.remove(&connection_id) {
            connection.stop_connection();
        }

        if let Some((id, mut stream)) = self.write_list.remove(&connection_id) {
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
    }

    pub async fn write_frame(&self, connection_id: u64, resp: MQTTPacketWrapper) {
        let mut times = 0;
        let cluster = self.cache_manager.get_cluster_info();
        loop {
            match self.write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if times > cluster.send_max_try_mut_times {
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
                    if times > cluster.send_max_try_mut_times {
                        error(format!("[write_frame]Connection management could not obtain an available connection. Connection ID: {},len:{}",connection_id,self.write_list.len()));
                        break;
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > cluster.send_max_try_mut_times {
                        error(format!("[write_frame]Connection management failed to get connection variable reference, connection ID: {}",connection_id));
                        break;
                    }
                }
            }
            times = times + 1;
            sleep(Duration::from_millis(cluster.send_try_mut_sleep_time_ms)).await
        }
    }

    pub fn tcp_connect_num_check(&self) -> bool {
        let cluster = self.cache_manager.get_cluster_info();
        if self.connections.len() >= cluster.tcp_max_connection_num as usize {
            return true;
        }
        return false;
    }

    pub fn get_connect(&self, connect_id: u64) -> Option<NetworkConnection> {
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
