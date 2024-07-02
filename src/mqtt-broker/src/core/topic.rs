use crate::core::{cache_manager::CacheManager, error::MQTTBrokerError};
use crate::storage::topic::TopicStorage;
use bytes::Bytes;
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use common_base::tools::unique_id;
use metadata_struct::mqtt::topic::MQTTTopic;
use protocol::mqtt::common::{Publish, PublishProperties};
use regex::Regex;
use std::sync::Arc;
use storage_adapter::storage::{ShardConfig, StorageAdapter};

use super::connection::Connection;

pub const SYSTEM_TOPIC_BROKERS: &str = "$SYS/brokers";
pub const SYSTEM_TOPIC_BROKERS_VERSION: &str = "$SYS/brokers/${node}/version";
pub const SYSTEM_TOPIC_BROKERS_UPTIME: &str = "$SYS/brokers/${node}/uptime";
pub const SYSTEM_TOPIC_BROKERS_DATETIME: &str = "$SYS/brokers/${node}/datetime";
pub const SYSTEM_TOPIC_BROKERS_SYSDESCR: &str = "$SYS/brokers/${node}/sysdescr";
pub const SYSTEM_TOPIC_BROKERS_CLIENTS: &str = "$SYS/brokers/${node}/clients";

pub fn is_system_topic(topic_name: String) -> bool {
    return true;
}

pub fn payload_format_validator(
    payload: &Bytes,
    payload_format_indicator: u8,
    max_packet_size: usize,
) -> bool {
    if payload.len() == 0 || payload.len() > max_packet_size {
        return false;
    }

    if payload_format_indicator == 1 {
        return std::str::from_utf8(payload.to_vec().as_slice()).is_ok();
    }

    return false;
}

pub fn topic_name_validator(topic_name: &String) -> Result<(), MQTTBrokerError> {
    if topic_name.is_empty() {
        return Err(MQTTBrokerError::TopicNameIsEmpty);
    }
    let topic_slice: Vec<&str> = topic_name.split("/").collect();
    if topic_slice.first().unwrap().to_string() == "/".to_string() {
        return Err(MQTTBrokerError::TopicNameIncorrectlyFormatted);
    }

    if topic_slice.last().unwrap().to_string() == "/".to_string() {
        return Err(MQTTBrokerError::TopicNameIncorrectlyFormatted);
    }

    let format_str = "^[A-Za-z0-9+#/]+$".to_string();
    let re = Regex::new(&format!("{}", format_str)).unwrap();
    if !re.is_match(&topic_name) {
        return Err(MQTTBrokerError::TopicNameIncorrectlyFormatted);
    }
    return Ok(());
}

pub fn get_topic_name(
    connect_id: u64,
    publish: Publish,
    metadata_cache: Arc<CacheManager>,
    publish_properties: Option<PublishProperties>,
) -> Result<String, RobustMQError> {
    let topic_alias = if let Some(pub_properties) = publish_properties {
        pub_properties.topic_alias
    } else {
        None
    };

    if publish.topic.is_empty() && topic_alias.is_none() {
        return Err(RobustMQError::TopicNameInvalid());
    }

    let topic_name = if publish.topic.is_empty() {
        if let Some(tn) = metadata_cache.get_topic_alias(connect_id, topic_alias.unwrap()) {
            tn
        } else {
            return Err(RobustMQError::TopicNameInvalid());
        }
    } else {
        match String::from_utf8(publish.topic.to_vec()) {
            Ok(da) => da,
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        }
    };

    match topic_name_validator(&topic_name) {
        Ok(_) => {}
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }

    return Ok(topic_name);
}

pub fn save_topic_alias(
    connect_id: u64,
    topic_name: Bytes,
    cache_manager: Arc<CacheManager>,
    publish_properties: Option<PublishProperties>,
) -> Result<(), MQTTBrokerError> {
    if topic_name.is_empty() {
        return Ok(());
    }

    let topic = match String::from_utf8(topic_name.to_vec()) {
        Ok(da) => da,
        Err(e) => return Err(MQTTBrokerError::CommmonError(e.to_string())),
    };

    if topic.is_empty() {
        return Ok(());
    }

    if let Some(properties) = publish_properties {
        if let Some(alias) = properties.topic_alias {
            let cluster = cache_manager.get_cluster_info();
            if alias > cluster.topic_alias_max {
                return Err(MQTTBrokerError::TopicAliasTooLong);
            }

            if let Some(current_topic) = cache_manager.get_topic_alias(connect_id, alias) {
                // update mapping for topic&alias
                if current_topic != topic {
                    cache_manager.add_topic_alias(connect_id, topic, alias);
                }
            } else {
                cache_manager.add_topic_alias(connect_id, topic, alias);
            }
        }
    }
    return Ok(());
}

pub async fn get_topic_info<S>(
    topic_name: String,
    metadata_cache: Arc<CacheManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
) -> Result<MQTTTopic, RobustMQError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let topic = if let Some(tp) = metadata_cache.get_topic_by_name(&topic_name) {
        tp
    } else {
        let topic_storage = TopicStorage::new(client_poll.clone());
        let topic_id = unique_id();
        let topic = MQTTTopic::new(topic_id, topic_name.clone());
        match topic_storage.save_topic(topic.clone()).await {
            Ok(topic_id) => topic_id,
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        };
        metadata_cache.add_topic(&topic_name, &topic);

        // Create the resource object of the storage layer
        let shard_name = topic.topic_id.clone();
        let shard_config = ShardConfig::default();
        match message_storage_adapter
            .create_shard(shard_name, shard_config)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
        return Ok(topic);
    };
    return Ok(topic);
}

#[cfg(test)]
mod test {
    use crate::core::error::MQTTBrokerError;

    use super::topic_name_validator;

    #[test]
    pub fn topic_name_validator_test() {
        let topic_name = "".to_string();
        match topic_name_validator(&topic_name) {
            Ok(_) => {}
            Err(e) => {
                assert!(e.to_string() == MQTTBrokerError::TopicNameIsEmpty.to_string())
            }
        }

        let topic_name = "/test/test".to_string();
        match topic_name_validator(&topic_name) {
            Ok(_) => {}
            Err(e) => {
                assert!(e.to_string() == MQTTBrokerError::TopicNameIncorrectlyFormatted.to_string())
            }
        }

        let topic_name = "test/test/".to_string();
        match topic_name_validator(&topic_name) {
            Ok(_) => {}
            Err(e) => {
                assert!(e.to_string() == MQTTBrokerError::TopicNameIncorrectlyFormatted.to_string())
            }
        }

        let topic_name = "test/$1".to_string();
        match topic_name_validator(&topic_name) {
            Ok(_) => {}
            Err(e) => {
                assert!(e.to_string() == MQTTBrokerError::TopicNameIncorrectlyFormatted.to_string())
            }
        }

        let topic_name = "test/1".to_string();
        match topic_name_validator(&topic_name) {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}
