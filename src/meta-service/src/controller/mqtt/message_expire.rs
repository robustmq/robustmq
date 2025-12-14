// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::storage::keys::{
    storage_key_mqtt_last_will_prefix, storage_key_mqtt_retain_message_prefix,
};
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
use crate::storage::mqtt::topic::MqttTopicStorage;
use common_base::{error::common::CommonError, tools::now_second, utils::serialize};
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use metadata_struct::mqtt::retain_message::MQTTRetainMessage;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_META_DATA;
use rocksdb_engine::warp::StorageDataWrap;
use std::sync::Arc;
use tracing::error;

pub struct MessageExpire {
    cluster_name: String,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MessageExpire {
    pub fn new(cluster_name: String, rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MessageExpire {
            cluster_name,
            rocksdb_engine_handler,
        }
    }

    pub async fn retain_message_expire(&self) {
        let search_key = storage_key_mqtt_retain_message_prefix();
        let topic_storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());

        let cf = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_META_DATA)
        {
            cf
        } else {
            error!(
                "{}",
                CommonError::RocksDBFamilyNotAvailable(DB_COLUMN_FAMILY_META_DATA.to_string())
            );
            return;
        };

        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(search_key.clone());
        while iter.valid() {
            let key = iter.key();
            let value = iter.value();

            if key.is_none() || value.is_none() {
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
            let data = serialize::deserialize::<StorageDataWrap<MQTTRetainMessage>>(&result_value)
                .unwrap();
            let value = data.data;
            let delete = now_second() >= (value.create_time + value.retain_message_expired_at);
            if delete {
                if let Err(e) =
                    topic_storage.delete_retain_message(&self.cluster_name, &value.topic_name)
                {
                    error!("{}", e);
                }
            }

            iter.next();
        }
    }

    pub async fn last_will_message_expire(&self) {
        let search_key = storage_key_mqtt_last_will_prefix(&self.cluster_name);
        let lastwill_storage = MqttLastWillStorage::new(self.rocksdb_engine_handler.clone());

        let cf: std::sync::Arc<rocksdb::BoundColumnFamily<'_>> = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_META_DATA)
        {
            cf
        } else {
            error!(
                "{}",
                CommonError::RocksDBFamilyNotAvailable(DB_COLUMN_FAMILY_META_DATA.to_string())
            );
            return;
        };

        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(search_key.clone());
        while iter.valid() {
            let key = iter.key();
            let value = iter.value();

            if key.is_none() || value.is_none() {
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
            let data =
                serialize::deserialize::<StorageDataWrap<MqttLastWillData>>(&result_value).unwrap();
            let value = data.data;
            if let Some(properties) = value.last_will_properties {
                let delete = if let Some(expiry_interval) = properties.message_expiry_interval {
                    now_second() >= ((expiry_interval as u64) + data.create_time)
                } else {
                    now_second() >= ((86400 * 30) + data.create_time)
                };

                if delete {
                    if let Err(e) = lastwill_storage.delete(&self.cluster_name, &value.client_id) {
                        error!("{}", e);
                    }
                }
            }

            iter.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Bytes;
    use common_base::tools::{now_second, unique_id};
    use common_base::utils::file_utils::test_temp_dir;
    use metadata_struct::mqtt::lastwill::MqttLastWillData;
    use metadata_struct::mqtt::retain_message::MQTTRetainMessage;
    use metadata_struct::mqtt::session::MqttSession;
    use protocol::mqtt::common::LastWillProperties;
    use rocksdb_engine::rocksdb::RocksDBEngine;
    use rocksdb_engine::storage::family::column_family_list;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;

    use super::MessageExpire;
    use crate::storage::mqtt::lastwill::MqttLastWillStorage;
    use crate::storage::mqtt::session::MqttSessionStorage;
    use crate::storage::mqtt::topic::MqttTopicStorage;

    #[tokio::test]
    async fn retain_message_expire_test() {
        let cluster_name = unique_id();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            1000,
            column_family_list(),
        ));
        let message_expire =
            MessageExpire::new(cluster_name.clone(), rocksdb_engine_handler.clone());
        let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
        let topic_name = unique_id();
        let expired = 10;
        let retain_message = MQTTRetainMessage {
            cluster_name: cluster_name.clone(),
            topic_name: topic_name.clone(),
            retain_message: Bytes::from("message1"),
            retain_message_expired_at: expired,
            create_time: now_second(),
        };

        topic_storage.save_retain_message(retain_message).unwrap();

        tokio::spawn(async move {
            loop {
                message_expire.retain_message_expire().await;
                sleep(Duration::from_millis(1000)).await;
            }
        });

        let start = now_second();

        loop {
            let res = topic_storage
                .get_retain_message(&cluster_name, &topic_name)
                .unwrap();

            if res.is_none() {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }

        let ms = now_second() - start;
        println!("ms:{}", ms);
        assert!((9..=11).contains(&ms))
    }

    #[tokio::test]
    async fn last_will_message_expire_test() {
        let cluster_name = unique_id();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            1000,
            column_family_list(),
        ));
        let lastwill_storage = MqttLastWillStorage::new(rocksdb_engine_handler.clone());
        let session_storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());

        let client_id = unique_id();
        let last_will_properties = LastWillProperties {
            message_expiry_interval: Some(3),
            ..Default::default()
        };
        let last_will_message = MqttLastWillData {
            client_id: client_id.clone(),
            last_will: None,
            last_will_properties: Some(last_will_properties),
        };
        let message_expire =
            MessageExpire::new(cluster_name.clone(), rocksdb_engine_handler.clone());
        tokio::spawn(async move {
            loop {
                message_expire.last_will_message_expire().await;
                sleep(Duration::from_millis(1000)).await;
            }
        });

        let session = MqttSession {
            client_id: client_id.clone(),
            ..Default::default()
        };
        session_storage
            .save(&cluster_name, &client_id, session)
            .unwrap();
        lastwill_storage
            .save(&cluster_name, &client_id, last_will_message)
            .unwrap();

        let start = now_second();
        loop {
            let res = lastwill_storage.get(&cluster_name, &client_id).unwrap();
            if res.is_none() {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }

        let ms = now_second() - start;
        assert!(ms == 3 || ms == 4);
    }
}
