use crate::storage::rocksdb::RocksDBEngine;
use std::sync::Arc;

use super::{
    retain_message_expire::start_retain_message_expire_check, session_expire::SessionExpire,
};

pub struct MQTTController {
    session_expire: SessionExpire,
}

impl MQTTController {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> MQTTController {
        return MQTTController {
            session_expire: SessionExpire::new(rocksdb_engine_handler),
        };
    }

    pub async fn start(&self) {
        start_retain_message_expire_check().await;
        self.session_expire.start().await;
    }
}
