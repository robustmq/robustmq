// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::Message;
use bytes::BytesMut;
use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::cluster::MqttClusterDynamicConfig;
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::{DisconnectReasonCode, MqttProtocol};
use serde::{Deserialize, Serialize};
use tokio::select;
use tokio::sync::broadcast::{self};
use tokio::time::sleep;
use tracing::{debug, error, info};

use super::cache::{CacheManager, ConnectionLiveTime};
use super::connection::disconnect_connection;
use super::response::response_packet_mqtt_distinct_by_reason;
use crate::handler::error::MqttBrokerError;
use crate::server::connection_manager::ConnectionManager;
use crate::subscribe::manager::SubscribeManager;

pub struct ClientKeepAlive {
    cache_manager: Arc<CacheManager>,
    stop_send: broadcast::Sender<bool>,
    client_pool: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    subscribe_manager: Arc<SubscribeManager>,
}

impl ClientKeepAlive {
    pub fn new(
        client_pool: Arc<ClientPool>,
        connection_manager: Arc<ConnectionManager>,
        subscribe_manager: Arc<SubscribeManager>,
        cache_manager: Arc<CacheManager>,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        ClientKeepAlive {
            client_pool,
            connection_manager,
            subscribe_manager,
            cache_manager,
            stop_send,
        }
    }

    pub async fn start_heartbeat_check(&mut self) {
        loop {
            let mut stop_rx = self.stop_send.subscribe();
            select! {
                val = stop_rx.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            info!("{}","Heartbeat check thread stopped successfully.");
                            break;
                        }
                    }
                }
                val = self.keep_alive()=>{
                    if let Err(e) = val {
                        error!("{}",e);
                    }
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn keep_alive(&self) -> Result<(), MqttBrokerError> {
        let expire_connection = self.get_expire_connection().await;

        for connect_id in expire_connection {
            if let Some(connection) = self.cache_manager.get_connection(connect_id) {
                if let Some(network) = self.connection_manager.get_connect(connect_id) {
                    let protocol = network.protocol.clone().unwrap();
                    let resp = response_packet_mqtt_distinct_by_reason(
                        &protocol,
                        Some(DisconnectReasonCode::KeepAliveTimeout),
                    );

                    let wrap = MqttPacketWrapper {
                        protocol_version: protocol.clone().into(),
                        packet: resp,
                    };

                    if network.is_tcp() {
                        self.connection_manager
                            .write_tcp_frame(connection.connect_id, wrap)
                            .await?;

                        disconnect_connection(
                            &connection.client_id,
                            connect_id,
                            &self.cache_manager,
                            &self.client_pool,
                            &self.connection_manager,
                            &self.subscribe_manager,
                            false,
                        )
                        .await?;
                    } else {
                        let mut codec = MqttCodec::new(Some(protocol.into()));
                        let mut buff = BytesMut::new();
                        if let Err(e) = codec.encode_data(wrap.clone(), &mut buff) {
                            return Err(MqttBrokerError::WebsocketEncodePacketFailed(
                                e.to_string(),
                            ));
                        }

                        self.connection_manager
                            .write_websocket_frame(
                                connection.connect_id,
                                wrap,
                                Message::Binary(buff.to_vec()),
                            )
                            .await?;
                        disconnect_connection(
                            &connection.client_id,
                            connect_id,
                            &self.cache_manager,
                            &self.client_pool,
                            &self.connection_manager,
                            &self.subscribe_manager,
                            false,
                        )
                        .await?;
                    }
                    info!(
                        "Heartbeat timeout, active disconnection {} successful",
                        connect_id
                    );
                }
            }
        }

        for (client_id, _) in self.cache_manager.heartbeat_data.clone() {
            if !self.cache_manager.session_info.contains_key(&client_id) {
                self.cache_manager.heartbeat_data.remove(&client_id);
            }
        }
        Ok(())
    }

    async fn get_expire_connection(&self) -> Vec<u64> {
        let mut expire_connection = Vec::new();
        for (connect_id, connection) in self.cache_manager.connection_info.clone() {
            if let Some(time) = self.cache_manager.heartbeat_data.get(&connection.client_id) {
                let max_timeout = keep_live_time(time.keep_live) as u64;
                let now = now_second();
                if (now - time.heartbeat) >= max_timeout {
                    debug!("{},client_id:{},now:{},heartbeat:{}","Connection was closed by the server because the heartbeat timeout was not reported.",connection.client_id,now,time.heartbeat);
                    expire_connection.push(connect_id);
                }
            } else {
                let live_time = ConnectionLiveTime {
                    protocol: MqttProtocol::Mqtt5,
                    keep_live: connection.keep_alive,
                    heartbeat: now_second(),
                };
                self.cache_manager
                    .report_heartbeat(connection.client_id, live_time);
            }
        }
        expire_connection
    }
}

pub fn keep_live_time(keep_alive: u16) -> u16 {
    let new_keep_alive: u32 = (keep_alive as u32) * 3;
    if new_keep_alive > 65535 {
        return 65535;
    }
    new_keep_alive as u16
}

pub fn client_keep_live_time(cluster: &MqttClusterDynamicConfig, mut keep_alive: u16) -> u16 {
    if keep_alive == 0 {
        keep_alive = cluster.protocol.default_server_keep_alive;
    }
    if keep_alive > cluster.protocol.max_server_keep_alive {
        keep_alive = cluster.protocol.max_server_keep_alive / 3;
    }
    keep_alive
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct KeepAliveRunInfo {
    pub start_time: u128,
    pub end_time: u128,
    pub use_time: u128,
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::time::Duration;

    use common_base::config::broker_mqtt::BrokerMqttConfig;
    use common_base::tools::{local_hostname, now_second, unique_id};
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::cluster::MqttClusterDynamicConfig;
    use metadata_struct::mqtt::connection::{ConnectionConfig, MQTTConnection};
    use metadata_struct::mqtt::session::MqttSession;
    use tokio::sync::broadcast;
    use tokio::time::sleep;

    use super::keep_live_time;
    use crate::handler::cache::CacheManager;
    use crate::handler::keep_alive::{client_keep_live_time, ClientKeepAlive};
    use crate::server::connection_manager::ConnectionManager;
    use crate::subscribe::manager::SubscribeManager;

    #[tokio::test]
    pub async fn keep_live_test() {
        let mut config = MqttClusterDynamicConfig::default();
        config.protocol.default_server_keep_alive = 60;
        config.protocol.max_server_keep_alive = 300;

        let keep_alive = 0;
        let client_live = client_keep_live_time(&config, keep_alive);
        assert_eq!(client_live, 60);
        assert_eq!(keep_live_time(client_live), 180);

        let keep_alive = 50;
        let client_live = client_keep_live_time(&config, keep_alive);
        assert_eq!(client_live, 50);
        assert_eq!(keep_live_time(client_live), 150);

        let keep_alive = 100;
        let client_live = client_keep_live_time(&config, keep_alive);
        assert_eq!(client_live, 100);
        assert_eq!(keep_live_time(client_live), 300);

        let keep_alive = 500;
        let client_live = client_keep_live_time(&config, keep_alive);
        assert_eq!(client_live, 100);
        assert_eq!(keep_live_time(client_live), 300);
    }

    #[tokio::test]
    pub async fn client_keep_live_time_test() {
        let mut config = MqttClusterDynamicConfig::default();
        config.protocol.default_server_keep_alive = 60;
        config.protocol.max_server_keep_alive = 300;

        assert_eq!(client_keep_live_time(&config, 0), 60);
        assert_eq!(client_keep_live_time(&config, 400), 100);
        assert_eq!(client_keep_live_time(&config, 50), 50);
    }

    #[tokio::test]
    pub async fn keep_live_time_test() {
        let res = keep_live_time(3);
        assert_eq!(res, 9);
        let res = keep_live_time(1000);
        assert_eq!(res, 3000);
        let res = keep_live_time(50000);
        assert_eq!(res, 65535);
    }

    #[tokio::test]
    pub async fn get_expire_connection_test() {
        let conf = BrokerMqttConfig {
            cluster_name: "test".to_string(),
            ..Default::default()
        };
        let client_pool = Arc::new(ClientPool::new(100));
        let (stop_send, _) = broadcast::channel::<bool>(2);

        let cache_manager = Arc::new(CacheManager::new(
            client_pool.clone(),
            conf.cluster_name.clone(),
        ));

        let connection_manager = Arc::new(ConnectionManager::new(cache_manager.clone()));
        let subscribe_manager = Arc::new(SubscribeManager::new());
        let alive = ClientKeepAlive::new(
            client_pool,
            connection_manager,
            subscribe_manager,
            cache_manager.clone(),
            stop_send,
        );

        let client_id = unique_id();
        let connect_id = 1;

        let session = MqttSession::new(client_id.clone(), 60, false, None);
        cache_manager.add_session(&client_id, &session);

        let keep_alive = 2;
        let addr = local_hostname();
        let config = ConnectionConfig {
            connect_id: 1,
            client_id,
            receive_maximum: 100,
            max_packet_size: 100,
            topic_alias_max: 100,
            request_problem_info: 100,
            keep_alive,
            source_ip_addr: addr,
        };
        let connection = MQTTConnection::new(config);
        cache_manager.add_connection(connect_id, connection);

        let start = now_second();
        loop {
            let res = alive.get_expire_connection().await;
            if res.contains(&connect_id) {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
        assert_eq!((now_second() - start), keep_live_time(keep_alive) as u64);
    }
}
