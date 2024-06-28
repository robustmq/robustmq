use crate::{cache::placement::PlacementCacheManager, storage::rocksdb::RocksDBEngine};
use message_expire::MessageExpire;
use session_expire::SessionExpire;
use std::sync::Arc;

pub mod message_expire;
pub mod session_expire;

pub struct MQTTController {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    placement_center_cache: Arc<PlacementCacheManager>,
}

impl MQTTController {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        placement_center_cache: Arc<PlacementCacheManager>,
    ) -> MQTTController {
        return MQTTController {
            rocksdb_engine_handler,
            placement_center_cache,
        };
    }

    pub fn start(&self) {
        // Periodically check if the session has expired
        let session = SessionExpire::new(self.rocksdb_engine_handler.clone());
        tokio::spawn(async move {
            session.session_expire().await;
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
}
