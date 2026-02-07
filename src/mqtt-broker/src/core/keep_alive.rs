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
use crate::core::connection::build_server_disconnect_conn_context;
use crate::core::error::MqttBrokerError;
use crate::mqtt::disconnect::build_distinct_packet;
use crate::subscribe::manager::SubscribeManager;
use axum::extract::ws::Message;
use bytes::BytesMut;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
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
use tracing::{debug, info, warn};

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
        loop_select_ticket(ac_fn, 1000, &self.stop_send).await;
        info!("Heartbeat check thread stopped successfully.");
    }

    async fn keep_alive(&self) -> ResultCommonError {
        let config = self.cache_manager.broker_cache.get_cluster_config().await;
        if !config.mqtt_keep_alive.enable {
            return Ok(());
        }

        let expire_connection = self.get_expire_connection().await;

        for connect_id in expire_connection {
            if let Some(connection) = self.cache_manager.get_connection(connect_id) {
                if let Some(network) = self.connection_manager.get_connect(connect_id) {
                    let protocol = network.protocol.clone().unwrap();
                    let resp = build_distinct_packet(
                        &self.cache_manager,
                        connect_id,
                        &protocol.to_mqtt(),
                        Some(DisconnectReasonCode::NormalDisconnection),
                        None,
                        Some("keep alive timeout".to_string()),
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
                    tokio::spawn(async move {
                        if let Err(e) = try_send_distinct_packet(&context).await {
                            warn!(
                                connect_id = context.connect_id,
                                client_id = %context.connection.client_id,
                                protocol = ?context.protocol,
                                error = %e,
                                "Heartbeat timeout: failed to actively disconnect connection"
                            );
                        } else {
                            debug!(
                                "Heartbeat timeout, active disconnection {} successful",
                                context.connect_id
                            );
                        }
                    });
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
                let max_timeout = keep_live_time(&self.cache_manager, time.keep_live).await as u64;
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

async fn try_send_distinct_packet(
    context: &TrySendDistinctPacketContext,
) -> Result<(), MqttBrokerError> {
    let close_conn_fn = async move || -> Result<(), MqttBrokerError> {
        if context.network.is_tcp() {
            context
                .connection_manager
                .write_tcp_frame(
                    context.connection.connect_id,
                    RobustMQPacketWrapper::from_mqtt(context.wrap.clone()),
                )
                .await?;
        } else if context.network.is_quic() {
            context
                .connection_manager
                .write_quic_frame(
                    context.connection.connect_id,
                    RobustMQPacketWrapper::from_mqtt(context.wrap.clone()),
                )
                .await?;
        } else {
            let mut codec = MqttCodec::new(Some(context.protocol.clone().into()));
            let mut buff = BytesMut::new();
            codec.encode_data(context.wrap.clone(), &mut buff)?;
            context
                .connection_manager
                .write_websocket_frame(
                    context.connection.connect_id,
                    RobustMQPacketWrapper::from_mqtt(context.wrap.clone()),
                    Message::Binary(buff.to_vec().into()),
                )
                .await?
        }
        Ok(())
    };

    if let Err(e) = close_conn_fn().await {
        warn!(
            connect_id = context.connect_id,
            client_id = %context.connection.client_id,
            protocol = ?context.protocol,
            network = ?context.network,
            error = %e,
            "Failed to send keep-alive disconnect packet"
        );
    }

    let context = build_server_disconnect_conn_context(
        &context.cache_manager,
        &context.client_pool,
        &context.connection_manager,
        &context.subscribe_manager,
        context.connect_id,
        &context.protocol,
    )?;
    disconnect_connection(context).await?;
    record_mqtt_connection_expired();
    Ok(())
}

pub async fn keep_live_time(cache_manager: &Arc<MQTTCacheManager>, keep_alive: u16) -> u16 {
    let config = cache_manager.broker_cache.get_cluster_config().await;
    keep_alive * config.mqtt_keep_alive.default_timeout
}

pub async fn client_keep_live_time(
    cache_manager: &Arc<MQTTCacheManager>,
    mut keep_alive: u16,
) -> u16 {
    let config = cache_manager.broker_cache.get_cluster_config().await;
    if keep_alive == 0 {
        keep_alive = config.mqtt_keep_alive.default_time;
    }
    if keep_alive > config.mqtt_keep_alive.max_time {
        keep_alive = config.mqtt_keep_alive.max_time / config.mqtt_keep_alive.default_timeout;
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
    use super::keep_live_time;
    use crate::core::keep_alive::{client_keep_live_time, ClientKeepAlive};
    use crate::core::tool::test_build_mqtt_cache_manager;
    use crate::subscribe::manager::SubscribeManager;
    use common_base::tools::{local_hostname, now_second};

    use common_base::uuid::unique_id;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::connection::{ConnectionConfig, MQTTConnection};
    use metadata_struct::mqtt::session::MqttSession;
    use network_server::common::connection_manager::ConnectionManager;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::broadcast;
    use tokio::time::sleep;

    #[tokio::test]
    pub async fn keep_live_test() {
        let cache_manager = test_build_mqtt_cache_manager().await;

        let keep_alive = 0;
        let client_live = client_keep_live_time(&cache_manager, keep_alive).await;
        assert_eq!(client_live, 180);
        assert_eq!(keep_live_time(&cache_manager, client_live).await, 360);

        let keep_alive = 50;
        let client_live = client_keep_live_time(&cache_manager, keep_alive).await;
        assert_eq!(client_live, 50);
        assert_eq!(keep_live_time(&cache_manager, client_live).await, 100);

        let keep_alive = 100;
        let client_live = client_keep_live_time(&cache_manager, keep_alive).await;
        assert_eq!(client_live, 100);
        assert_eq!(keep_live_time(&cache_manager, client_live).await, 200);

        let keep_alive = 500;
        let client_live = client_keep_live_time(&cache_manager, keep_alive).await;
        assert_eq!(client_live, 500);
        assert_eq!(keep_live_time(&cache_manager, client_live).await, 1000);

        let keep_alive = 4000;
        let client_live = client_keep_live_time(&cache_manager, keep_alive).await;
        assert_eq!(client_live, 1800);
        assert_eq!(keep_live_time(&cache_manager, client_live).await, 3600);
    }

    #[tokio::test]
    pub async fn get_expire_connection_test() {
        let client_pool = Arc::new(ClientPool::new(100));
        let (stop_send, _) = broadcast::channel::<bool>(2);
        let cache_manager = test_build_mqtt_cache_manager().await;
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

        let session = MqttSession::new(client_id.clone(), 60, false, None, true);
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
            clean_session: false,
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
        assert_eq!(
            (now_second() - start),
            keep_live_time(&cache_manager, keep_alive).await as u64
        );
    }
}
