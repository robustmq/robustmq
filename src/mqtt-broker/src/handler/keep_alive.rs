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

use super::cache::{ConnectionLiveTime, MQTTCacheManager};
use super::connection::disconnect_connection;
use super::response::response_packet_mqtt_distinct_by_reason;
use crate::subscribe::manager::SubscribeManager;
use axum::extract::ws::Message;
use bytes::BytesMut;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select, now_second};
use common_config::config::BrokerConfig;
use common_metrics::mqtt::event::record_mqtt_connection_expired;
use grpc_clients::pool::ClientPool;
use metadata_struct::connection::NetworkConnection;
use metadata_struct::mqtt::connection::MQTTConnection;
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::{DisconnectReasonCode, MqttProtocol};
use protocol::robust::RobustMQPacketWrapper;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast::{self};
use tracing::{debug, info};

#[derive(Clone)]
pub struct TrySendDistinctPacketContext {
    pub cache_manager: Arc<MQTTCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub connection_manager: Arc<ConnectionManager>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub network: NetworkConnection,
    pub connection: MQTTConnection,
    pub wrap: MqttPacketWrapper,
    pub protocol: MqttProtocol,
    pub connect_id: u64,
}

#[derive(Clone)]
pub struct ClientKeepAlive {
    cache_manager: Arc<MQTTCacheManager>,
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
        cache_manager: Arc<MQTTCacheManager>,
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

    pub async fn start_heartbeat_check(&self) {
        let ac_fn = async || -> ResultCommonError { self.keep_alive().await };
        loop_select(ac_fn, 1, &self.stop_send).await;
        info!("Heartbeat check thread stopped successfully.");
    }

    async fn keep_alive(&self) -> ResultCommonError {
        let expire_connection = self.get_expire_connection().await;

        for connect_id in expire_connection {
            if let Some(connection) = self.cache_manager.get_connection(connect_id) {
                if let Some(network) = self.connection_manager.get_connect(connect_id) {
                    let protocol = network.protocol.clone().unwrap();
                    let resp = response_packet_mqtt_distinct_by_reason(
                        &protocol.to_mqtt(),
                        Some(DisconnectReasonCode::NormalDisconnection),
                    );

                    let wrap = MqttPacketWrapper {
                        protocol_version: protocol.to_u8(),
                        packet: resp,
                    };

                    let context = TrySendDistinctPacketContext {
                        cache_manager: self.cache_manager.clone(),
                        client_pool: self.client_pool.clone(),
                        connection_manager: self.connection_manager.clone(),
                        subscribe_manager: self.subscribe_manager.clone(),
                        network: network.clone(),
                        connection: connection.clone(),
                        wrap,
                        protocol: protocol.to_mqtt(),
                        connect_id,
                    };
                    self.try_send_distinct_packet(context);
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

    fn try_send_distinct_packet(&self, context: TrySendDistinctPacketContext) {
        tokio::spawn(async move {
            if context.network.is_tcp() {
                let _ = context
                    .connection_manager
                    .write_tcp_frame(
                        context.connection.connect_id,
                        RobustMQPacketWrapper::from_mqtt(context.wrap.clone()),
                    )
                    .await;
            } else if context.network.is_quic() {
                let _ = context
                    .connection_manager
                    .write_quic_frame(
                        context.connection.connect_id,
                        RobustMQPacketWrapper::from_mqtt(context.wrap.clone()),
                    )
                    .await;
            } else {
                let mut codec = MqttCodec::new(Some(context.protocol.clone().into()));
                let mut buff = BytesMut::new();
                if codec.encode_data(context.wrap.clone(), &mut buff).is_err() {
                    return;
                }

                let _ = context
                    .connection_manager
                    .write_websocket_frame(
                        context.connection.connect_id,
                        RobustMQPacketWrapper::from_mqtt(context.wrap.clone()),
                        Message::Binary(buff.to_vec()),
                    )
                    .await;
            }

            let _ = disconnect_connection(
                &context.connection.client_id,
                context.connect_id,
                &context.cache_manager,
                &context.client_pool,
                &context.connection_manager,
                &context.subscribe_manager,
                false,
            )
            .await;
            record_mqtt_connection_expired();
            debug!(
                "Heartbeat timeout, active disconnection {} successful",
                context.connect_id
            );
        });
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

pub fn client_keep_live_time(cluster: &BrokerConfig, mut keep_alive: u16) -> u16 {
    if keep_alive == 0 {
        keep_alive = cluster.mqtt_protocol_config.default_server_keep_alive;
    }
    if keep_alive > cluster.mqtt_protocol_config.max_server_keep_alive {
        keep_alive = cluster.mqtt_protocol_config.max_server_keep_alive / 3;
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

    use common_base::tools::{local_hostname, now_second, unique_id};
    use common_config::config::BrokerConfig;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::connection::{ConnectionConfig, MQTTConnection};
    use metadata_struct::mqtt::session::MqttSession;
    use tokio::sync::broadcast;
    use tokio::time::sleep;

    use super::keep_live_time;
    use crate::common::tool::test_build_mqtt_cache_manager;
    use crate::handler::keep_alive::{client_keep_live_time, ClientKeepAlive};
    use crate::subscribe::manager::SubscribeManager;
    use network_server::common::connection_manager::ConnectionManager;

    #[tokio::test]
    pub async fn keep_live_test() {
        let mut config = BrokerConfig::default();
        config.mqtt_protocol_config.default_server_keep_alive = 60;
        config.mqtt_protocol_config.max_server_keep_alive = 300;

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
        let mut config = BrokerConfig::default();
        config.mqtt_protocol_config.default_server_keep_alive = 60;
        config.mqtt_protocol_config.max_server_keep_alive = 300;

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
        let client_pool = Arc::new(ClientPool::new(100));
        let (stop_send, _) = broadcast::channel::<bool>(2);
        let cache_manager = test_build_mqtt_cache_manager();
        let connection_manager = Arc::new(ConnectionManager::new(3, 1000));
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
