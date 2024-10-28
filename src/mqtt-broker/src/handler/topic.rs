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
use common_base::error::common::CommonError;
use common_base::error::mqtt_broker::MqttBrokerError;
use common_base::tools::unique_id;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::MqttTopic;
use protocol::mqtt::common::{Publish, PublishProperties};
use regex::Regex;
use storage_adapter::storage::{ShardConfig, StorageAdapter};

use crate::handler::cache::CacheManager;
use crate::storage::topic::TopicStorage;

pub fn is_system_topic(_: String) -> bool {
    true
}

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
    connect_id: u64,
    metadata_cache: &Arc<CacheManager>,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> Result<String, MqttBrokerError> {
    let topic_alias = if let Some(pub_properties) = publish_properties {
        pub_properties.topic_alias
    } else {
        None
    };

    let topic = String::from_utf8(publish.topic.to_vec())?;

    if topic.is_empty() && topic_alias.is_none() {
        return Err(MqttBrokerError::TopicNameIsEmpty);
    }

    let topic_name = if topic.is_empty() {
        if let Some(tn) = metadata_cache.get_topic_alias(connect_id, topic_alias.unwrap()) {
            tn
        } else {
            return Err(MqttBrokerError::TopicNameInvalid());
        }
    } else {
        topic
    };
    topic_name_validator(&topic_name)?;
    Ok(topic_name)
}

pub async fn try_init_topic<S>(
    topic_name: &str,
    metadata_cache: &Arc<CacheManager>,
    message_storage_adapter: &Arc<S>,
    client_pool: &Arc<ClientPool>,
) -> Result<MqttTopic, CommonError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let topic = if let Some(tp) = metadata_cache.get_topic_by_name(topic_name) {
        tp
    } else {
        let topic_storage = TopicStorage::new(client_pool.clone());
        let topic_id = unique_id();
        let topic = MqttTopic::new(topic_id, topic_name.to_owned());
        topic_storage.save_topic(topic.clone()).await?;
        metadata_cache.add_topic(topic_name, &topic);

        // Create the resource object of the storage layer
        let shard_name = topic.topic_id.clone();
        let shard_config = ShardConfig::default();
        message_storage_adapter
            .create_shard(shard_name, shard_config)
            .await?;
        return Ok(topic);
    };
    Ok(topic)
}

#[cfg(test)]
mod test {

    use common_base::error::mqtt_broker::MqttBrokerError;

    use super::topic_name_validator;

    #[test]
    pub fn topic_name_validator_test() {
        let topic_name = "".to_string();
        if let Err(e) = topic_name_validator(&topic_name) {
            // assert!(e.to_string() == MqttBrokerError::TopicNameIsEmpty.to_string())
            assert_eq!(e, MqttBrokerError::TopicNameIsEmpty);
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
}
