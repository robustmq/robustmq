use super::cache_manager::CacheManager;
use crate::{
    metrics::metrics_heartbeat_keep_alive_run_info, server::tcp::packet::RequestPackage,
    subscribe::subscribe_cache::SubscribeCacheManager,
};
use common_base::{
    log::{debug, error, info},
    tools::{now_mills, now_second},
};
use protocol::mqtt::common::{
    Disconnect, DisconnectProperties, DisconnectReasonCode, MQTTPacket, MQTTProtocol,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
    select,
    sync::broadcast::{self, Sender},
};

pub struct ClientKeepAlive {
    cache_manager: Arc<CacheManager>,
    sucscribe_cache: Arc<SubscribeCacheManager>,
    response_queue_sx: Sender<RequestPackage>,
    stop_send: broadcast::Sender<bool>,
}

impl ClientKeepAlive {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        sucscribe_cache: Arc<SubscribeCacheManager>,
        response_queue_sx: Sender<RequestPackage>,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        return ClientKeepAlive {
            cache_manager,
            sucscribe_cache,
            response_queue_sx,
            stop_send,
        };
    }

    // TCP connection heartbeat detection is performed in parallel, and subsequent processing is carried out
    pub async fn start_heartbeat_check(&mut self) {
        loop {
            let mut stop_rx = self.stop_send.subscribe();
            select! {
                val = stop_rx.recv() =>{
                    match val{
                        Ok(flag) => {
                            if flag {
                                info("TCP Server acceptor thread stopped successfully.".to_string());
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
        let start_time = now_mills();
        for (client_id, time) in self.cache_manager.heartbeat_data.clone() {
            // The server will decide that the connection has failed twice as long as the client-set expiration time.
            let max_timeout = keep_live_time(time.keep_live);
            if (now_second() - time.heartbeat) > max_timeout {
                if let Some(connect_id) = self.cache_manager.get_connect_id(client_id.clone()) {
                    let disconnect = Disconnect {
                        reason_code: DisconnectReasonCode::KeepAliveTimeout,
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

                    match self.response_queue_sx.send(response) {
                        Ok(_) => {}
                        Err(e) => {
                            error(e.to_string());
                        }
                    };
                } else {
                    self.cache_manager.remove_heartbeat(&client_id);
                }
            }
        }

        let end_time = now_mills();
        let use_time = end_time - start_time;
        let run_info = KeepAliveRunInfo {
            start_time,
            end_time,
            use_time,
        };
        metrics_heartbeat_keep_alive_run_info(use_time);
        debug(format!("{:?}", run_info));
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
