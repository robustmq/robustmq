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

use bytes::Bytes;

use common_base::tools::unique_id;
use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::MqttTopic;
use protocol::mqtt::common::{Publish, PublishProperties};
use regex::Regex;
use storage_adapter::storage::{ShardInfo, StorageAdapter};

use super::error::MqttBrokerError;
use crate::handler::cache::CacheManager;
use crate::handler::topic_rewrite::convert_publish_topic_by_rewrite_rule;
use crate::storage::message::cluster_name;
use crate::storage::topic::TopicStorage;

pub fn payload_format_validator(
    payload: &Bytes,
    payload_format_indicator: u8,
    max_packet_size: usize,
) -> bool {
    if payload.is_empty() || payload.len() > max_packet_size {
        return false;
    }

    if payload_format_indicator == 1 {
        return std::str::from_utf8(payload.to_vec().as_slice()).is_ok();
    }

    false
}

pub fn topic_name_validator(topic_name: &str) -> Result<(), MqttBrokerError> {
    if topic_name.is_empty() {
        return Err(MqttBrokerError::TopicNameIsEmpty);
    }

    let topic_slice: Vec<&str> = topic_name.split("/").collect();
    if topic_slice.first().unwrap() == &"/" {
        return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(
            topic_name.to_owned(),
        ));
    }

    if topic_slice.last().unwrap() == &"/" {
        return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(
            topic_name.to_owned(),
        ));
    }

    let format_str = "^[A-Za-z0-9_+#/$]+$";
    let re = Regex::new(format_str).unwrap();
    if !re.is_match(topic_name) {
        return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(
            topic_name.to_owned(),
        ));
    }
    Ok(())
}

pub fn get_topic_name(
    cache_manager: &Arc<CacheManager>,
    connect_id: u64,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> Result<String, MqttBrokerError> {
    let topic = String::from_utf8(publish.topic.to_vec())?;

    let topic_alias = if let Some(pub_properties) = publish_properties {
        pub_properties.topic_alias
    } else {
        None
    };

    if topic.is_empty() && topic_alias.is_none() {
        return Err(MqttBrokerError::TopicNameIsEmpty);
    }

    let topic_name = if topic.is_empty() {
        if let Some(tn) = cache_manager.get_topic_alias(connect_id, topic_alias.unwrap()) {
            tn
        } else {
            return Err(MqttBrokerError::TopicNameInvalid());
        }
    } else {
        topic
    };

    topic_name_validator(&topic_name)?;

    // topic rewrite
    if let Some(rewrite_topic_name) =
        convert_publish_topic_by_rewrite_rule(cache_manager, &topic_name)?
    {
        topic_name_validator(rewrite_topic_name.as_str())?;
        return Ok(rewrite_topic_name);
    }

    Ok(topic_name)
}

pub async fn try_init_topic<S>(
    topic_name: &str,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
    client_pool: &Arc<ClientPool>,
) -> Result<MqttTopic, MqttBrokerError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let topic = if let Some(tp) = metadata_cache.get_topic_by_name(topic_name) {
        tp
    } else {
        let namespace = cluster_name();

        // create Topic
        let topic_storage = TopicStorage::new(client_pool.clone());
        let topic_id = unique_id();
        let conf = broker_mqtt_conf();
        let topic = if let Some(topic) = topic_storage.get_topic(topic_name).await? {
            topic
        } else {
            let topic = MqttTopic::new(topic_id, conf.cluster_name.clone(), topic_name.to_owned());
            topic_storage.save_topic(topic.clone()).await?;
            topic
        };
        metadata_cache.add_topic(topic_name, &topic);

        // Create the resource object of the storage layer
        let list = message_storage_adapter
            .list_shard(namespace.clone(), topic_name.to_owned())
            .await?;
        if list.is_empty() {
            let shard = ShardInfo {
                namespace: namespace.clone(),
                shard_name: topic_name.to_owned(),
                replica_num: 1,
            };
            message_storage_adapter.create_shard(shard).await?;
        }
        return Ok(topic);
    };
    Ok(topic)
}

#[cfg(test)]
mod test {
    use super::topic_name_validator;
    use crate::handler::error::MqttBrokerError;

    #[test]
    pub fn topic_name_validator_test() {
        let topic_name = "".to_string();
        if let Err(e) = topic_name_validator(&topic_name) {
            // assert!(e.to_string() == MqttBrokerError::TopicNameIsEmpty.to_string())
            assert_eq!(e.to_string(), MqttBrokerError::TopicNameIsEmpty.to_string());
        }

        let topic_name = "/test/test".to_string();
        topic_name_validator(&topic_name).unwrap();

        let topic_name = "test/test/".to_string();
        topic_name_validator(&topic_name).unwrap();

        let topic_name = "test/$1".to_string();
        if let Err(err) = topic_name_validator(&topic_name) {
            println!("{:?}", err);
        }

        let topic_name = "test/1".to_string();
        topic_name_validator(&topic_name).unwrap();

        let topic_name =
            "/sys/request_response/response/1eb1f833e0de4169908acedec8eb62f7".to_string();
        topic_name_validator(&topic_name).unwrap();
    }

    #[test]
    pub fn gen_rewrite_topic_test() {}
}
