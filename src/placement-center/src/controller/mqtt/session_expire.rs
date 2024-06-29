use crate::{
    cache::{mqtt::MqttCacheManager, placement::PlacementCacheManager},
    storage::{keys::storage_key_mqtt_session_cluster_prefix, rocksdb::RocksDBEngine},
};
use clients::poll::ClientPool;
use common_base::tools::now_second;
use metadata_struct::mqtt::session::MQTTSession;
use std::{sync::Arc, thread::sleep, time::Duration};
use super::call_broker::MQTTBrokerCall;

pub struct ExpireLastWill {
    pub client_id: String,
    pub delay_sec: u64,
    pub cluster_name: String,
}

pub struct SessionExpire {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    mqtt_cache_manager: Arc<MqttCacheManager>,
    placement_cache_manager: Arc<PlacementCacheManager>,
    client_poll: Arc<ClientPool>,
}

impl SessionExpire {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        mqtt_cache_manager: Arc<MqttCacheManager>,
        placement_cache_manager: Arc<PlacementCacheManager>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        return SessionExpire {
            rocksdb_engine_handler,
            mqtt_cache_manager,
            placement_cache_manager,
            client_poll,
        };
    }

    pub async fn session_expire(&self, cluster_name: String) {
        let search_key = storage_key_mqtt_session_cluster_prefix(&cluster_name);
        loop {
            let cf = self.rocksdb_engine_handler.cf_mqtt();
            let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(cf);
            iter.seek(search_key.clone());
            let mut sessions = Vec::new();
            while iter.valid() {
                let key = iter.key();
                let value = iter.value();

                if key == None || value == None {
                    continue;
                }

                let result_key = match String::from_utf8(key.unwrap().to_vec()) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                if !result_key.starts_with(&search_key) {
                    break;
                }

                let result_value = value.unwrap().to_vec();
                let session = serde_json::from_slice::<MQTTSession>(&result_value).unwrap();
                if self.is_expire(&session) {
                    sessions.push(session);
                }
            }
            if sessions.len() > 0 {
                let call = MQTTBrokerCall::new(
                    cluster_name.clone(),
                    self.placement_cache_manager.clone(),
                    self.rocksdb_engine_handler.clone(),
                    self.client_poll.clone(),
                    self.mqtt_cache_manager.clone(),
                );
                tokio::spawn(async move {
                    call.delete_sessions(sessions).await;
                });
            }
            sleep(Duration::from_secs(1));
        }
    }

    fn is_expire(&self, session: &MQTTSession) -> bool {
        if session.connection_id.is_none() && session.broker_id.is_none() {
            if let Some(distinct_time) = session.distinct_time {
                if now_second() >= (session.session_expiry + distinct_time) {
                    return true;
                }
            }
        }

        return false;
    }
}
