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

use std::{sync::Arc, time::Duration};

use common_base::{error::common::CommonError, tools::now_second};
use log::error;
use metadata_struct::mqtt::{lastwill::LastWillData, topic::MQTTTopic};
use tokio::time::sleep;

use crate::storage::{
    keys::{storage_key_mqtt_last_will_prefix, storage_key_mqtt_topic_cluster_prefix},
    mqtt::{lastwill::MQTTLastWillStorage, topic::MQTTTopicStorage},
    rocksdb::{RocksDBEngine, DB_COLUMN_FAMILY_CLUSTER},
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
                    match topic_storage.save(&self.cluster_name, &value.topic_name.clone(), value) {
                        Ok(()) => {}
                        Err(e) => {
                            error!("{}", e);
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
            if let Some(properties) = value.last_will_properties {
                let delete = if let Some(expiry_interval) = properties.message_expiry_interval {
                    now_second() >= ((expiry_interval as u64) + data.create_time)
                } else {
                    now_second() >= ((86400 * 30) + data.create_time)
                };

                if delete {
                    match lastwill_storage.delete(&self.cluster_name, &value.client_id) {
                        Ok(()) => {}
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                }
            }

            iter.next();
        }
        sleep(Duration::from_secs(1)).await;
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::{
        mqtt::{
            lastwill::MQTTLastWillStorage, session::MQTTSessionStorage, topic::MQTTTopicStorage,
        },
        rocksdb::{column_family_list, RocksDBEngine},
    };
    use common_base::{
        config::placement_center::placement_center_test_conf,
        tools::{now_second, unique_id},
    };
    use metadata_struct::mqtt::{
        lastwill::LastWillData, message::MQTTMessage, session::MQTTSession, topic::MQTTTopic,
    };
    use protocol::mqtt::common::{LastWillProperties, Publish};
    use std::{fs::remove_dir_all, sync::Arc, time::Duration};
    use tokio::time::sleep;

    use super::MessageExpire;

    #[tokio::test]
    async fn retain_message_expire_test() {
        let config = placement_center_test_conf();

        let cluster_name = unique_id();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let message_expire =
            MessageExpire::new(cluster_name.clone(), rocksdb_engine_handler.clone());

        let topic_storage = MQTTTopicStorage::new(rocksdb_engine_handler.clone());
        let topic = MQTTTopic::new(unique_id(), "tp1".to_string());
        topic_storage
            .save(&cluster_name, &topic.topic_name.clone(), topic.clone())
            .unwrap();

        let retain_msg =
            MQTTMessage::build_message(&"c1".to_string(), &Publish::default(), &None, 600);
        topic_storage
            .set_topic_retain_message(
                &cluster_name,
                &topic.topic_name.clone(),
                retain_msg.encode(),
                3,
            )
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

        remove_dir_all(config.rocksdb.data_path).unwrap();
    }

    #[tokio::test]
    async fn last_will_message_expire_test() {
        let config = placement_center_test_conf();

        let cluster_name = unique_id();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let lastwill_storage = MQTTLastWillStorage::new(rocksdb_engine_handler.clone());
        let session_storage = MQTTSessionStorage::new(rocksdb_engine_handler.clone());

        let client_id = unique_id();
        let mut last_will_properties = LastWillProperties::default();
        last_will_properties.message_expiry_interval = Some(3);
        let last_will_message = LastWillData {
            client_id: client_id.clone(),
            last_will: None,
            last_will_properties: Some(last_will_properties),
        };
        let message_expire =
            MessageExpire::new(cluster_name.clone(), rocksdb_engine_handler.clone());
        tokio::spawn(async move {
            loop {
                message_expire.last_will_message_expire().await;
            }
        });

        let mut session = MQTTSession::default();
        session.client_id = client_id.clone();
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

        remove_dir_all(config.rocksdb.data_path).unwrap();
    }
}
