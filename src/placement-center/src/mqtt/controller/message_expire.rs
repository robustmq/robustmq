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

use std::sync::Arc;

use common_base::error::common::CommonError;
use common_base::tools::now_second;
use log::error;
use metadata_struct::mqtt::lastwill::LastWillData;
use metadata_struct::mqtt::topic::MqttTopic;
use rocksdb_engine::warp::StorageDataWrap;
use tokio::time::{self, Duration, Interval};

use crate::storage::keys::{
    storage_key_mqtt_last_will_prefix, storage_key_mqtt_topic_cluster_prefix,
};
use crate::storage::mqtt::lastwill::MqttLastWillStorage;
use crate::storage::mqtt::topic::MqttTopicStorage;
use crate::storage::rocksdb::{RocksDBEngine, DB_COLUMN_FAMILY_CLUSTER};

pub struct MessageExpire {
    cluster_name: String,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    interval: Interval,
}

impl MessageExpire {
    pub fn new(cluster_name: String, rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let intervarl = time::interval(Duration::from_secs(1));
        MessageExpire {
            cluster_name,
            rocksdb_engine_handler,
            interval: intervarl,
        }
    }

    pub async fn retain_message_expire(&mut self) {
        self.interval.tick().await;
        let search_key = storage_key_mqtt_topic_cluster_prefix(&self.cluster_name);
        let topic_storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());

        let cf = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_CLUSTER)
        {
            cf
        } else {
            error!(
                "{}",
                CommonError::RocksDBFamilyNotAvailable(DB_COLUMN_FAMILY_CLUSTER.to_string())
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
            let data = serde_json::from_slice::<StorageDataWrap>(&result_value).unwrap();
            let mut value = serde_json::from_str::<MqttTopic>(&data.data).unwrap();

            if value.retain_message.is_some() {
                let delete = if let Some(expired_at) = value.retain_message_expired_at {
                    now_second() >= (data.create_time + expired_at)
                } else {
                    false
                };
                if delete {
                    value.retain_message = None;
                    value.retain_message_expired_at = None;
                    if let Err(e) =
                        topic_storage.save(&self.cluster_name, &value.topic_name.clone(), value)
                    {
                        error!("{}", e);
                    }
                }
            }
            iter.next();
        }
    }

    pub async fn last_will_message_expire(&mut self) {
        self.interval.tick().await;
        let search_key = storage_key_mqtt_last_will_prefix(&self.cluster_name);
        let lastwill_storage = MqttLastWillStorage::new(self.rocksdb_engine_handler.clone());

        let cf: std::sync::Arc<rocksdb::BoundColumnFamily<'_>> = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_CLUSTER)
        {
            cf
        } else {
            error!(
                "{}",
                CommonError::RocksDBFamilyNotAvailable(DB_COLUMN_FAMILY_CLUSTER.to_string())
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
            let data = serde_json::from_slice::<StorageDataWrap>(&result_value).unwrap();
            let value = serde_json::from_str::<LastWillData>(&data.data).unwrap();
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
    use common_base::tools::{now_second, unique_id};
    use common_base::utils::file_utils::test_temp_dir;
    use metadata_struct::mqtt::lastwill::LastWillData;
    use metadata_struct::mqtt::message::MqttMessage;
    use metadata_struct::mqtt::session::MqttSession;
    use metadata_struct::mqtt::topic::MqttTopic;
    use protocol::mqtt::common::{LastWillProperties, Publish};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;

    use super::MessageExpire;
    use crate::storage::mqtt::lastwill::MqttLastWillStorage;
    use crate::storage::mqtt::session::MqttSessionStorage;
    use crate::storage::mqtt::topic::MqttTopicStorage;
    use crate::storage::rocksdb::{column_family_list, RocksDBEngine};

    #[tokio::test]
    async fn retain_message_expire_test() {
        let cluster_name = unique_id();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            1000,
            column_family_list(),
        ));
        let mut message_expire =
            MessageExpire::new(cluster_name.clone(), rocksdb_engine_handler.clone());
        let topic_storage = MqttTopicStorage::new(rocksdb_engine_handler.clone());
        let mut topic = MqttTopic::new(unique_id(), "t1".to_string(), "tp1".to_string());
        let retain_msg = MqttMessage::build_message("c1", &Publish::default(), &None, 600);
        topic.retain_message = Some(retain_msg.encode());
        topic.retain_message_expired_at = Some(3);

        topic_storage
            .save(&cluster_name, &topic.topic_name.clone(), topic.clone())
            .unwrap();
        tokio::spawn(async move {
            loop {
                message_expire.retain_message_expire().await;
            }
        });

        let start = now_second();
        loop {
            let res = topic_storage.get(&cluster_name, &topic.topic_name).unwrap();
            let tp = res.unwrap();
            if tp.retain_message.is_none() {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }

        let ms = now_second() - start;
        assert!(ms == 3 || ms == 4);
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
        let last_will_message = LastWillData {
            client_id: client_id.clone(),
            last_will: None,
            last_will_properties: Some(last_will_properties),
        };
        let mut message_expire =
            MessageExpire::new(cluster_name.clone(), rocksdb_engine_handler.clone());
        tokio::spawn(async move {
            loop {
                message_expire.last_will_message_expire().await;
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
