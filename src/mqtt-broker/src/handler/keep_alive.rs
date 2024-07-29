use super::{
    cache_manager::{CacheManager, ConnectionLiveTime},
    connection::disconnect_connection,
};
use crate::server::connection_manager::ConnectionManager;
use clients::poll::ClientPool;
use common_base::{
    log::{error, info},
    tools::now_second,
};
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
}

impl ClientKeepAlive {
    pub fn new(
        client_poll: Arc<ClientPool>,
        connnection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<CacheManager>,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        return ClientKeepAlive {
            client_poll,
            connnection_manager,
            cache_manager,
            stop_send,
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
                                info("Heartbeat check thread stopped successfully.".to_string());
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
        for (connect_id, connection) in self.cache_manager.connection_info.clone() {
            if let Some(time) = self.cache_manager.heartbeat_data.get(&connection.client_id) {
                let max_timeout = keep_live_time(time.keep_live);
                if (now_second() - time.heartbeat) > max_timeout {
                    info("Connection was closed by the server because the heartbeat timeout was not reported.".to_string());
                    match disconnect_connection(
                        &connection.client_id,
                        connect_id,
                        &self.cache_manager,
                        &self.client_poll,
                        &self.connnection_manager,
                    )
                    .await
                    {
                        Ok(()) => {}
                        Err(e) => {
                            error(e.to_string());
                        }
                    }
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
        for (client_id, _) in self.cache_manager.heartbeat_data.clone() {
            if !self.cache_manager.session_info.contains_key(&client_id) {
                self.cache_manager.heartbeat_data.remove(&client_id);
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
}

pub fn keep_live_time(keep_alive: u16) -> u64 {
    return (keep_alive * 2) as u64;
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct KeepAliveRunInfo {
    pub start_time: u128,
    pub end_time: u128,
    pub use_time: u128,
}

#[cfg(test)]
mod test {
    use crate::handler::cache_manager::CacheManager;
    use crate::handler::connection::Connection;
    use crate::handler::keep_alive::ClientKeepAlive;
    use crate::server::connection_manager::ConnectionManager;
    use clients::poll::ClientPool;
    use common_base::config::broker_mqtt::BrokerMQTTConfig;
    use common_base::tools::now_second;
    use metadata_struct::mqtt::session::MQTTSession;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    #[tokio::test]
    pub async fn keep_alive_test() {
        let mut conf = BrokerMQTTConfig::default();
        conf.cluster_name = "test".to_string();
        let client_poll = Arc::new(ClientPool::new(100));
        let (stop_send, _) = broadcast::channel::<bool>(2);

        let cache_manager = Arc::new(CacheManager::new(
            client_poll.clone(),
            conf.cluster_name.clone(),
        ));
        let connnection_manager = Arc::new(ConnectionManager::new(cache_manager.clone()));
        let mut keep_alive = ClientKeepAlive::new(
            client_poll,
            connnection_manager,
            cache_manager.clone(),
            stop_send,
        );
        tokio::spawn(async move {
            keep_alive.start_heartbeat_check().await;
        });

        let client_id = "clietn-id-123".to_string();
        let connect_id = 1;
        let session = MQTTSession::new(&client_id, 60, false, None);
        cache_manager.add_session(client_id.clone(), session);
        let start_time = now_second();
        let connection = Connection::new(1, &client_id, 100, 100, 100, 100, 1);
        cache_manager.add_connection(connect_id, connection);
    }
}
