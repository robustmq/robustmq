use crate::core::metadata_cache::MetadataCacheManager;
use crate::storage::topic::TopicStorage;
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use metadata_struct::mqtt::topic::MQTTTopic;
use protocol::mqtt::{Publish, PublishProperties};
use regex::Regex;
use std::sync::Arc;
use storage_adapter::storage::{ShardConfig, StorageAdapter};

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

pub fn publish_get_topic_name(
    connect_id: u64,
    publish: Publish,
    metadata_cache: Arc<MetadataCacheManager>,
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

    match topic_name_validator(topic_name.clone()) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    }

    return Ok(topic_name);
}

pub async fn get_topic_info<S>(
    topic_name: String,
    metadata_cache: Arc<MetadataCacheManager>,
    message_storage_adapter: Arc<S>,
    client_poll: Arc<ClientPool>,
) -> Result<MQTTTopic, RobustMQError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let topic = if let Some(tp) = metadata_cache.get_topic_by_name(topic_name.clone()) {
        tp
    } else {
        let topic_storage = TopicStorage::new(client_poll.clone());
        let topic = MQTTTopic::new(&topic_name);
        match topic_storage.save_topic(topic_name.clone()).await {
            Ok(topic_id) => topic_id,
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        };

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
    use common_base::errors::RobustMQError;

    use super::topic_name_validator;

    #[test]
    pub fn topic_name_validator_test() {
        let topic_name = "".to_string();
        match topic_name_validator(topic_name) {
            Ok(_) => {}
            Err(e) => {
                assert!(e.to_string() == RobustMQError::TopicNameIsEmpty.to_string())
            }
        }

        let topic_name = "/test/test".to_string();
        match topic_name_validator(topic_name) {
            Ok(_) => {}
            Err(e) => {
                assert!(e.to_string() == RobustMQError::TopicNameIncorrectlyFormatted.to_string())
            }
        }

        let topic_name = "test/test/".to_string();
        match topic_name_validator(topic_name) {
            Ok(_) => {}
            Err(e) => {
                assert!(e.to_string() == RobustMQError::TopicNameIncorrectlyFormatted.to_string())
            }
        }

        let topic_name = "test/$1".to_string();
        match topic_name_validator(topic_name) {
            Ok(_) => {}
            Err(e) => {
                assert!(e.to_string() == RobustMQError::TopicNameIncorrectlyFormatted.to_string())
            }
        }

        let topic_name = "test/1".to_string();
        match topic_name_validator(topic_name) {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}
