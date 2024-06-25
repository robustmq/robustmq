use common_base::tools::now_second;
use metadata_struct::mqtt::session::MQTTSession;

use crate::storage::{keys::storage_key_all_session_prefix, rocksdb::RocksDBEngine};
use core::slice::SlicePattern;
use std::sync::Arc;

pub struct SessionExpire {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SessionExpire {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        return SessionExpire {
            rocksdb_engine_handler,
        };
    }

    pub async fn start(&self) {
        let search_key = storage_key_all_session_prefix();
        loop {
            let cf = self.rocksdb_engine_handler.cf_mqtt();
            let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(cf);
            iter.seek(search_key.clone());
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
                if now_second() <= session.session_expiry {
                    // call broker delete session
                    // delete local session
                }
            }
        }
    }
}
