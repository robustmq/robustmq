// Copyright 2023 RobustMQ Team
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

use super::{
    cache::{CacheManager, ConnectionLiveTime},
    connection::disconnect_connection,
};
use crate::{
    server::connection_manager::ConnectionManager, subscribe::subscribe_manager::SubscribeManager,
};
use clients::poll::ClientPool;
use common_base::tools::now_second;
use log::{error, info};
use metadata_struct::mqtt::cluster::MQTTClusterDynamicConfig;
use protocol::mqtt::common::MQTTProtocol;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::{
    select,
    sync::broadcast::{self},
    time::sleep,
};

pub struct ClientKeepAlive {
    cache_manager: Arc<CacheManager>,
    stop_send: broadcast::Sender<bool>,
    client_poll: Arc<ClientPool>,
    connnection_manager: Arc<ConnectionManager>,
    subscribe_manager: Arc<SubscribeManager>,
}

impl ClientKeepAlive {
    pub fn new(
        client_poll: Arc<ClientPool>,
        subscribe_manager: Arc<SubscribeManager>,
        connnection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<CacheManager>,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        return ClientKeepAlive {
            client_poll,
            connnection_manager,
            cache_manager,
            stop_send,
            subscribe_manager,
        };
    }

    pub async fn start_heartbeat_check(&mut self) {
        loop {
            let mut stop_rx = self.stop_send.subscribe();
            select! {
                val = stop_rx.recv() =>{
                    match val{
                        Ok(flag) => {
                            if flag {
                                info!("{}","Heartbeat check thread stopped successfully.");
                                break;
                            }
                        }
                        Err(_) => {}
                    }
                }
                _ = self.keep_alive()=>{
                }
            }
        }
    }

    async fn keep_alive(&self) {
        let expire_connection = self.get_expire_connection().await;

        for connect_id in expire_connection {
            if let Some(connection) = self.cache_manager.get_connection(connect_id) {
                match disconnect_connection(
                    &connection.client_id,
                    connect_id,
                    &self.cache_manager,
                    &self.client_poll,
                    &self.connnection_manager,
                    &self.subscribe_manager,
                )
                .await
                {
                    Ok(()) => {}
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }

        for (client_id, _) in self.cache_manager.heartbeat_data.clone() {
            if !self.cache_manager.session_info.contains_key(&client_id) {
                self.cache_manager.heartbeat_data.remove(&client_id);
            }
        }
        sleep(Duration::from_secs(1)).await;
    }

    async fn get_expire_connection(&self) -> Vec<u64> {
        let mut expire_connection = Vec::new();
        for (connect_id, connection) in self.cache_manager.connection_info.clone() {
            if let Some(time) = self.cache_manager.heartbeat_data.get(&connection.client_id) {
                let max_timeout = keep_live_time(time.keep_live) as u64;
                if (now_second() - time.heartbeat) >= max_timeout {
                    info!("{}","Connection was closed by the server because the heartbeat timeout was not reported.");
                    expire_connection.push(connect_id);
                }
            } else {
                let live_time = ConnectionLiveTime {
                    protobol: MQTTProtocol::MQTT5,
                    keep_live: connection.keep_alive as u16,
                    heartbeat: now_second(),
                };
                self.cache_manager
                    .report_heartbeat(&connection.client_id, live_time);
            }
        }
        return expire_connection;
    }
}

pub fn keep_live_time(keep_alive: u16) -> u16 {
    let new_keep_alive: u32 = (keep_alive as u32) * 2;
    if new_keep_alive > 65535 {
        return 65535;
    }
    return new_keep_alive as u16;
}

pub fn client_keep_live_time(cluster: &MQTTClusterDynamicConfig, mut keep_alive: u16) -> u16 {
    if keep_alive <= 0 {
        keep_alive = cluster.protocol.default_server_keep_alive;
    }
    if keep_alive > cluster.protocol.max_server_keep_alive {
        keep_alive = cluster.protocol.max_server_keep_alive / 2;
    }
    return keep_alive;
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
    use crate::handler::cache::CacheManager;
    use crate::handler::connection::Connection;
    use crate::handler::keep_alive::ClientKeepAlive;
    use crate::server::connection_manager::ConnectionManager;
    use crate::subscribe::subscribe_manager::SubscribeManager;
    use clients::poll::ClientPool;
    use common_base::config::broker_mqtt::BrokerMQTTConfig;
    use common_base::tools::{now_second, unique_id};
    use metadata_struct::mqtt::session::MQTTSession;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::broadcast;
    use tokio::time::sleep;

    #[tokio::test]
    pub async fn keep_live_time_test() {
        let res = keep_live_time(3);
        assert_eq!(res, 6);
        let res = keep_live_time(1000);
        assert_eq!(res, 2000);
        let res = keep_live_time(50000);
        assert_eq!(res, 65535);
    }

    #[tokio::test]
    pub async fn get_expire_connection_test() {
        let mut conf = BrokerMQTTConfig::default();
        conf.cluster_name = "test".to_string();
        let client_poll = Arc::new(ClientPool::new(100));
        let (stop_send, _) = broadcast::channel::<bool>(2);

        let cache_manager = Arc::new(CacheManager::new(
            client_poll.clone(),
            conf.cluster_name.clone(),
        ));

        let subscribe_manager = Arc::new(SubscribeManager::new(
            cache_manager.clone(),
            client_poll.clone(),
        ));

        let connnection_manager = Arc::new(ConnectionManager::new(cache_manager.clone()));
        let alive = ClientKeepAlive::new(
            client_poll,
            subscribe_manager,
            connnection_manager,
            cache_manager.clone(),
            stop_send,
        );

        let client_id = unique_id();
        let connect_id = 1;

        let session = MQTTSession::new(&client_id, 60, false, None);
        cache_manager.add_session(client_id.clone(), session);

        let keep_alive = 2;
        let addr = "127.0.0.1".to_string();
        let connection = Connection::new(1, &client_id, 100, 100, 100, 100, keep_alive, addr);
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
            keep_live_time(keep_alive as u16) as u64
        );
    }
}
