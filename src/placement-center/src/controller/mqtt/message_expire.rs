use std::{sync::Arc, time::Duration};

use common_base::{
    log::{error, info},
    tools::now_second,
};
use metadata_struct::mqtt::{lastwill::LastWillData, topic::MQTTTopic};
use tokio::time::sleep;

use crate::storage::{
    keys::{storage_key_mqtt_last_will_prefix, storage_key_mqtt_topic_cluster_prefix},
    mqtt::{lastwill::MQTTLastWillStorage, topic::MQTTTopicStorage},
    rocksdb::RocksDBEngine,
    StorageDataWrap,
};

pub struct MessageExpire {
    cluster_name: String,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MessageExpire {
    pub fn new(cluster_name: String, rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        return MessageExpire {
            cluster_name,
            rocksdb_engine_handler,
        };
    }

    pub async fn retain_message_expire(&self) {
        let search_key = storage_key_mqtt_topic_cluster_prefix(&self.cluster_name);
        let topic_storage = MQTTTopicStorage::new(self.rocksdb_engine_handler.clone());

        let cf = self.rocksdb_engine_handler.cf_mqtt();
        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(cf);
        iter.seek(search_key.clone());
        while iter.valid() {
            let key = iter.key();
            let value = iter.value();

            if key == None || value == None {
                iter.next();
                continue;
            }
            let result_key = match String::from_utf8(key.unwrap().to_vec()) {
                Ok(s) => s,
                Err(_) => {
                    iter.next();
                    continue;
                }
            };

            if !result_key.starts_with(&search_key) {
                break;
            }

            let result_value = value.unwrap().to_vec();
            let data = serde_json::from_slice::<StorageDataWrap>(&result_value).unwrap();
            let mut value = serde_json::from_slice::<MQTTTopic>(data.data.as_slice()).unwrap();
            if !value.retain_message.is_none() {
                let delete = if let Some(expired_at) = value.retain_message_expired_at {
                    now_second() >= (data.create_time + expired_at)
                } else {
                    false
                };
                if delete {
                    value.retain_message = None;
                    value.retain_message_expired_at = None;
                    match topic_storage.save(&self.cluster_name, &value.topic_name, value.encode())
                    {
                        Ok(()) => {}
                        Err(e) => {
                            error(e.to_string());
                        }
                    }
                }
            }
            iter.next();
        }
        sleep(Duration::from_secs(1)).await;
    }

    pub async fn last_will_message_expire(&self) {
        let search_key = storage_key_mqtt_last_will_prefix(&self.cluster_name);
        let lastwill_storage = MQTTLastWillStorage::new(self.rocksdb_engine_handler.clone());

        let cf = self.rocksdb_engine_handler.cf_mqtt();
        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(cf);
        iter.seek(search_key.clone());
        while iter.valid() {
            let key = iter.key();
            let value = iter.value();

            if key == None || value == None {
                iter.next();
                continue;
            }
            let result_key = match String::from_utf8(key.unwrap().to_vec()) {
                Ok(s) => s,
                Err(_) => {
                    iter.next();
                    continue;
                }
            };

            if !result_key.starts_with(&search_key) {
                iter.next();
                break;
            }

            let result_value = value.unwrap().to_vec();
            let data = serde_json::from_slice::<StorageDataWrap>(&result_value).unwrap();
            let value = serde_json::from_slice::<LastWillData>(data.data.as_slice()).unwrap();
            if !value.last_will.is_none() {
                if let Some(properties) = value.last_will_properties {
                    let delete = if let Some(expiry_interval) = properties.message_expiry_interval {
                        now_second() >= ((expiry_interval as u64) + data.create_time)
                    } else {
                        true
                    };

                    if delete {
                        match lastwill_storage.delete_last_will_message(
                            self.cluster_name.clone(),
                            value.client_id.clone(),
                        ) {
                            Ok(()) => {}
                            Err(e) => {
                                error(e.to_string());
                            }
                        }
                    }
                }
            }
            iter.next();
        }
        sleep(Duration::from_secs(1)).await;
    }
}
