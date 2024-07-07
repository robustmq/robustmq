use super::{cache_manager::CacheManager, retain::save_topic_retain_message};
use crate::storage::{message::MessageStorage, session::SessionStorage};
use clients::poll::ClientPool;
use common_base::errors::RobustMQError;
use metadata_struct::mqtt::{cluster::MQTTCluster, lastwill::LastWillData, message::MQTTMessage};
use protocol::mqtt::common::{LastWill, LastWillProperties, Publish, PublishProperties};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

pub async fn send_last_will_message<S>(
    client_id: &String,
    cache_manager: &Arc<CacheManager>,
    client_poll: &Arc<ClientPool>,
    last_will: &Option<LastWill>,
    last_will_properties: &Option<LastWillProperties>,
    message_storage_adapter: Arc<S>,
) -> Result<(), RobustMQError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    if let Some(will) = last_will {
        if will.topic.is_empty() || will.message.is_empty() {
            return Ok(());
        }
        let topic_name = String::from_utf8(will.topic.to_vec()).unwrap();
        let topic = if let Some(tp) = cache_manager.get_topic_by_name(&topic_name) {
            tp
        } else {
            return Err(RobustMQError::TopicDoesNotExist(topic_name.clone()));
        };

        let publish = Publish {
            dup: false,
            qos: will.qos,
            pkid: 0,
            retain: will.retain,
            topic: will.topic.clone(),
            payload: will.message.clone(),
        };

        let properties = if let Some(properties) = last_will_properties {
            Some(PublishProperties {
                payload_format_indicator: properties.payload_format_indicator,
                message_expiry_interval: properties.message_expiry_interval,
                topic_alias: None,
                response_topic: properties.response_topic.clone(),
                user_properties: Vec::new(),
                subscription_identifiers: Vec::new(),
                correlation_data: properties.correlation_data.clone(),
                content_type: properties.content_type.clone(),
            })
        } else {
            None
        };
        // Persisting retain message data
        match save_topic_retain_message(
            cache_manager,
            client_poll,
            &topic_name,
            client_id,
            &publish,
            &properties,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                return Err(e);
            }
        }

        // Persisting stores message data
        let message_storage = MessageStorage::new(message_storage_adapter.clone());
        if let Some(record) = MQTTMessage::build_record(client_id, &publish, &properties) {
            match message_storage
                .append_topic_message(topic.topic_id.clone(), vec![record])
                .await
            {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
    return Ok(());
}

pub async fn save_last_will_message(
    client_id: String,
    last_will: Option<LastWill>,
    last_will_properties: Option<LastWillProperties>,
    client_poll: Arc<ClientPool>,
) -> Result<(), RobustMQError> {
    if last_will.is_none() {
        return Ok(());
    }

    let session_storage = SessionStorage::new(client_poll);
    let lastwill = LastWillData {
        client_id: client_id.clone(),
        last_will,
        last_will_properties,
    };
    return session_storage
        .save_last_will_messae(client_id, lastwill.encode())
        .await;
}

pub fn last_will_delay_interval(last_will_properties: &Option<LastWillProperties>) -> Option<u64> {
    let delay_interval = if let Some(properties) = last_will_properties.clone() {
        if let Some(value) = properties.delay_interval {
            value
        } else {
            return None;
        }
    } else {
        return None;
    };

    return Some(delay_interval as u64);
}

pub fn check_lastwill_payload(
    cluster: &MQTTCluster,
    last_will: &Option<LastWill>,
) -> (bool, usize) {
    if let Some(will) = last_will {
        if will.message.len() > (cluster.max_packet_size() as usize) {
            return (false, will.message.len());
        }
    }
    return (true, 0);
}
