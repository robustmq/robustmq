use crate::{
    cache::{mqtt::MqttCacheManager, placement::PlacementCacheManager},
    storage::rocksdb::RocksDBEngine,
};
use clients::poll::ClientPool;
use message_expire::MessageExpire;
use session_expire::SessionExpire;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

pub mod call_broker;
pub mod message_expire;
pub mod session_expire;

pub struct MQTTController {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    placement_center_cache: Arc<PlacementCacheManager>,
    mqtt_cache_manager: Arc<MqttCacheManager>,
    client_poll: Arc<ClientPool>,
}

impl MQTTController {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_center_cache: Arc<PlacementCacheManager>,
        mqtt_cache_manager: Arc<MqttCacheManager>,
        client_poll: Arc<ClientPool>,
    ) -> MQTTController {
        return MQTTController {
            rocksdb_engine_handler,
            placement_center_cache,
            mqtt_cache_manager,
            client_poll,
        };
    }

    pub fn start(&self) {
        loop {
            for (cluster_name, _) in self.placement_center_cache.cluster_list.clone() {
                // Periodically check if the session has expired
                let session = SessionExpire::new(
                    self.rocksdb_engine_handler.clone(),
                    self.mqtt_cache_manager.clone(),
                    self.placement_center_cache.clone(),
                    self.client_poll.clone(),
                );
                tokio::spawn(async move {
                    session.session_expire(cluster_name).await;
                });

                // Whether the timed message expires
                let message = MessageExpire::new();
                tokio::spawn(async move {
                    message.retain_message_expire().await;
                });

                // Periodically detects whether a will message is sent
                let message = MessageExpire::new();
                tokio::spawn(async move {
                    message.last_will_message_expire().await;
                });
            }
            sleep(Duration::from_secs(1));
        }
    }
}
