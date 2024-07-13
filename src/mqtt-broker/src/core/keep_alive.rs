use super::cache_manager::{CacheManager, ConnectionLiveTime};
use crate::server::tcp::packet::RequestPackage;
use common_base::{
    log::{error, info},
    tools::now_second,
};
use protocol::mqtt::common::{
    Disconnect, DisconnectProperties, DisconnectReasonCode, MQTTPacket, MQTTProtocol,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::{
    select,
    sync::broadcast::{self, Sender},
    time::sleep,
};

pub struct ClientKeepAlive {
    cache_manager: Arc<CacheManager>,
    request_queue_sx: Sender<RequestPackage>,
    stop_send: broadcast::Sender<bool>,
}

impl ClientKeepAlive {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        request_queue_sx: Sender<RequestPackage>,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        return ClientKeepAlive {
            cache_manager,
            request_queue_sx,
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
                    let disconnect = if time.protobol == MQTTProtocol::MQTT4
                        || time.protobol == MQTTProtocol::MQTT3
                    {
                        Disconnect {
                            reason_code: Some(DisconnectReasonCode::KeepAliveTimeout),
                        }
                    } else {
                        Disconnect { reason_code: None }
                    };

                    let response = if time.protobol == MQTTProtocol::MQTT4
                        || time.protobol == MQTTProtocol::MQTT3
                    {
                        RequestPackage {
                            connection_id: connect_id,
                            addr: "127.0.0.1:1000".parse().unwrap(),
                            packet: MQTTPacket::Disconnect(disconnect.clone(), None),
                        }
                    } else {
                        let properties = Some(DisconnectProperties {
                                                        session_expiry_interval: None,
                                                        reason_string: Some("Connection was closed by the server because the heartbeat timeout was not reported.".to_string()),
                                                        user_properties: vec![("heartbeat_close".to_string(), "true".to_string())],
                                                        server_reference: None,
                                                    });
                        RequestPackage {
                            connection_id: connect_id,
                            addr: "127.0.0.1:1000".parse().unwrap(),
                            packet: MQTTPacket::Disconnect(disconnect, properties),
                        }
                    };

                    match self.request_queue_sx.send(response) {
                        Ok(_) => {}
                        Err(e) => {
                            error(e.to_string());
                        }
                    };
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
    use std::sync::Arc;

    use clients::poll::ClientPool;
    use common_base::config::broker_mqtt::BrokerMQTTConfig;
    use common_base::tools::now_second;
    use metadata_struct::mqtt::session::MQTTSession;
    use protocol::mqtt::common::MQTTPacket;
    use tokio::sync::broadcast;

    use crate::core::cache_manager::CacheManager;
    use crate::core::connection::Connection;
    use crate::core::keep_alive::ClientKeepAlive;

    #[tokio::test]
    pub async fn keep_alive_test() {
        let mut conf = BrokerMQTTConfig::default();
        conf.cluster_name = "test".to_string();
        let client_poll = Arc::new(ClientPool::new(100));
        let (stop_send, _) = broadcast::channel::<bool>(2);

        let (request_queue_sx, mut request_queue_rx) = broadcast::channel(1000);
        let cache_manager = Arc::new(CacheManager::new(
            client_poll.clone(),
            conf.cluster_name.clone(),
        ));

        let mut keep_alive =
            ClientKeepAlive::new(cache_manager.clone(), request_queue_sx, stop_send.clone());
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

        match request_queue_rx.recv().await {
            Ok(da) => {
                stop_send.send(true).unwrap();
                let res = if let MQTTPacket::Disconnect(_, _) = da.packet {
                    true
                } else {
                    false
                };
                assert!(res);
                assert_eq!(now_second() - start_time, 3);
            }
            Err(_) => {
                assert!(false);
            }
        }
    }
}
