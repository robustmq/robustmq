use crate::{metrics::metrics_heartbeat_keep_alive_run_info, server::tcp::packet::RequestPackage};
use common_base::{
    log::{debug, error, info},
    tools::{now_mills, now_second},
};
use protocol::mqtt::common::{
    Disconnect, DisconnectProperties, DisconnectReasonCode, MQTTPacket, MQTTProtocol,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{
        broadcast::{self, Sender},
        Semaphore,
    },
    time::sleep,
};

use super::cache_manager::CacheManager;

pub struct ClientKeepAlive {
    shard_num: u64,
    cache_manager: Arc<CacheManager>,
    request_queue_sx: Sender<RequestPackage>,
    stop_send: broadcast::Receiver<bool>,
}

impl ClientKeepAlive {
    pub fn new(
        shard_num: u64,
        heartbeat_manager: Arc<CacheManager>,
        request_queue_sx: Sender<RequestPackage>,
        stop_send: broadcast::Receiver<bool>,
    ) -> Self {
        return ClientKeepAlive {
            shard_num,
            cache_manager: heartbeat_manager,
            request_queue_sx,
            stop_send,
        };
    }

    // TCP connection heartbeat detection is performed in parallel, and subsequent processing is carried out
    pub async fn start_heartbeat_check(&mut self) {
        loop {
            match self.stop_send.try_recv() {
                Ok(flag) => {
                    if flag {
                        info("KeepAlive thread stopped successfully".to_string());
                        break;
                    }
                }
                Err(_) => {}
            }

            sleep(Duration::from_secs(5)).await;

            let semaphore = Arc::new(Semaphore::new(self.shard_num as usize));
            for i in 0..self.shard_num {
                let data = self.cache_manager.get_shard_data(i);
                let request_queue_sx = self.request_queue_sx.clone();
                let sp = semaphore.clone();
                tokio::spawn(async move {
                    match sp.acquire().await {
                        Ok(_) => {}
                        Err(e) => {
                            error(format!("Heartbeat thread failed to retrieve the semaplight with error message:{}",e.to_string()));
                        }
                    }

                    for (connect_id, time) in data.heartbeat_data.clone() {
                        // The server will decide that the connection has failed twice as long as the client-set expiration time.
                        let max_timeout = (time.keep_live * 2) as u64;
                        if (now_second() - time.heartbeat) > max_timeout {
                            let disconnect = Disconnect {
                                reason_code: DisconnectReasonCode::KeepAliveTimeout,
                            };
                            let properties = Some(DisconnectProperties {
                                            session_expiry_interval: None,
                                            reason_string: Some("Connection was closed by the server because the heartbeat timeout was not reported.".to_string()),
                                            user_properties: vec![("heartbeat_close".to_string(), "true".to_string())],
                                            server_reference: None,
                                        });
                            let req = if time.protobol == MQTTProtocol::MQTT4
                                || time.protobol == MQTTProtocol::MQTT3
                            {
                                RequestPackage {
                                    connection_id: connect_id,
                                    addr: "127.0.0.1:1000".parse().unwrap(),
                                    packet: MQTTPacket::Disconnect(disconnect.clone(), None),
                                }
                            } else {
                                RequestPackage {
                                    connection_id: connect_id,
                                    addr: "127.0.0.1:1000".parse().unwrap(),
                                    packet: MQTTPacket::Disconnect(disconnect, properties),
                                }
                            };

                            match request_queue_sx.send(req) {
                                Ok(_) => {}
                                Err(e) => {
                                    error(e.to_string());
                                }
                            };
                        }
                    }
                });
            }

            // Waiting for all spawn to complete, thinking about the next batch of detection
            let start_time = now_mills();
            loop {
                if semaphore.available_permits() == (self.shard_num as usize) {
                    break;
                }
                sleep(Duration::from_millis(100)).await;
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
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct KeepAliveRunInfo {
    pub start_time: u128,
    pub end_time: u128,
    pub use_time: u128,
}
