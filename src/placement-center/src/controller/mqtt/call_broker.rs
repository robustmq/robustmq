use clients::{
    mqtt::call::{broker_mqtt_delete_session, send_last_will_message},
    poll::ClientPool,
};
use common_base::{
    log::{error, info, warn},
    tools::now_second,
};
use metadata_struct::mqtt::{lastwill::LastWillData, session::MQTTSession};
use protocol::broker_server::generate::mqtt::{DeleteSessionRequest, SendLastWillMessageRequest};

use crate::{
    cache::{mqtt::MqttCacheManager, placement::PlacementCacheManager},
    storage::{mqtt::session::MQTTSessionStorage, rocksdb::RocksDBEngine},
};
use std::sync::Arc;

use super::session_expire::ExpireLastWill;

pub struct MQTTBrokerCall {
    cluster_name: String,
    placement_cache_manager: Arc<PlacementCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    client_poll: Arc<ClientPool>,
    mqtt_cache_manager: Arc<MqttCacheManager>,
}

impl MQTTBrokerCall {
    pub fn new(
        cluster_name: String,
        placement_cache_manager: Arc<PlacementCacheManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        client_poll: Arc<ClientPool>,
        mqtt_cache_manager: Arc<MqttCacheManager>,
    ) -> Self {
        return MQTTBrokerCall {
            cluster_name,
            placement_cache_manager,
            rocksdb_engine_handler,
            client_poll,
            mqtt_cache_manager,
        };
    }

    pub async fn delete_sessions(&self, sessions: Vec<MQTTSession>) {
        let chunks: Vec<Vec<MQTTSession>> = sessions
            .chunks(100)
            .map(|chunk| chunk.to_vec()) // 将切片转换为Vec
            .collect();
        for raw in chunks {
            let client_ids: Vec<String> = raw.iter().map(|x| x.client_id.clone()).collect();
            let mut success = true;
            info(format!(
                "Session [{:?}] has expired. Call Broker to delete the Session information.",
                client_ids
            ));
            for addr in self
                .placement_cache_manager
                .get_cluster_node_addr(&self.cluster_name)
            {
                let request = DeleteSessionRequest {
                    client_id: client_ids.clone(),
                    cluster_name: self.cluster_name.clone(),
                };
                match broker_mqtt_delete_session(self.client_poll.clone(), vec![addr], request)
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        success = false;
                        warn(e.to_string());
                    }
                }
            }

            if success {
                let session_storage = MQTTSessionStorage::new(self.rocksdb_engine_handler.clone());
                for ms in raw {
                    match session_storage.delete(self.cluster_name.clone(), ms.client_id.clone()) {
                        Ok(()) => {
                            let delay = if let Some(delay) = ms.last_will_delay_interval {
                                delay
                            } else {
                                0
                            };
                            self.mqtt_cache_manager
                                .add_expire_last_will(ExpireLastWill {
                                    client_id: ms.client_id.clone(),
                                    delay_sec: now_second() + delay,
                                    cluster_name: self.cluster_name.clone(),
                                });
                        }
                        Err(e) => error(e.to_string()),
                    }
                }
            }
        }
    }

    pub async fn send_last_will_message(&self, client_id: String, lastwill: LastWillData) {
        let request = SendLastWillMessageRequest {
            client_id: client_id.clone(),
            last_will_message: lastwill.encode(),
        };
        match send_last_will_message(
            self.client_poll.clone(),
            self.placement_cache_manager
                .get_cluster_node_addr(&self.cluster_name),
            request,
        )
        .await
        {
            Ok(_) => self
                .mqtt_cache_manager
                .remove_expire_last_will(&self.cluster_name, &client_id),
            Err(e) => {
                error(e.to_string());
            }
        }
    }
}
