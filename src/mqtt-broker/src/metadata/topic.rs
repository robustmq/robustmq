use std::sync::Arc;

use bytes::Bytes;
use common_base::{
    errors::RobustMQError,
    tools::{now_mills, unique_id},
};
use protocol::mqtt::{PubAckReason, Publish, PublishProperties};
use regex::Regex;
use serde::{Deserialize, Serialize};
use storage_adapter::storage::{ShardConfig, StorageAdapter};

use crate::storage::topic::TopicStorage;

use super::cache::MetadataCacheManager;

#[derive(Clone, Serialize, Deserialize)]
pub struct Topic {
    pub topic_id: String,
    pub topic_name: String,
    pub create_time: u128,
}

impl Topic {
    pub fn new(topic_name: &String) -> Self {
        return Topic {
            topic_id: unique_id(),
            topic_name: topic_name.clone(),
            create_time: now_mills(),
        };
    }
}

pub fn topic_name_validator(topic_name: String) -> Result<(), RobustMQError> {
    if topic_name.is_empty() {
        return Err(RobustMQError::TopicNameIsEmpty);
    }
    let topic_slice: Vec<&str> = topic_name.split("/").collect();
    if topic_slice.first().unwrap().to_string() == "/".to_string() {
        return Err(RobustMQError::TopicNameIncorrectlyFormatted);
    }

    if topic_slice.last().unwrap().to_string() == "/".to_string() {
        return Err(RobustMQError::TopicNameIncorrectlyFormatted);
    }

    let format_str = "^[A-Za-z0-9+#/]+$".to_string();
    let re = Regex::new(&format!("{}", format_str)).unwrap();
    if !re.is_match(&topic_name) {
        return Err(RobustMQError::TopicNameIncorrectlyFormatted);
    }
    return Ok(());
}

pub fn publish_get_topic_name<T>(
    connect_id: u64,
    publish: Publish,
    metadata_cache: Arc<MetadataCacheManager<T>>,
    publish_properties: Option<PublishProperties>,
) -> Result<String, RobustMQError>
where
    T: StorageAdapter + Sync + Send + 'static + Clone,
{
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

    match topic_name_validator(topic_name.clone()) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    }

    return Ok(topic_name);
}

pub async fn get_topic_info<T, S>(
    topic_name: String,
    metadata_cache: Arc<MetadataCacheManager<T>>,
    metadata_storage_adapter: Arc<T>,
    message_storage_adapter: Arc<S>,
) -> Result<Topic, RobustMQError>
where
    T: StorageAdapter + Sync + Send + 'static + Clone,
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let topic = if let Some(tp) = metadata_cache.get_topic_by_name(topic_name.clone()) {
        tp
    } else {
        // Persisting the topic information
        let topic = Topic::new(&topic_name);
        metadata_cache.set_topic(&topic_name, &topic);
        let topic_storage = TopicStorage::new(metadata_storage_adapter.clone());
        match topic_storage.save_topic(&topic_name, &topic).await {
            Ok(_) => {}
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }

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
        topic
    };
    return Ok(topic);
}
