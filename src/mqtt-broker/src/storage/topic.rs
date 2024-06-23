use clients::{
    placement::mqtt::call::{
        placement_create_topic, placement_delete_topic, placement_list_topic,
        placement_set_topic_retain_message,
    },
    poll::ClientPool,
};
use common_base::{config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError};
use dashmap::DashMap;
use metadata_struct::mqtt::{message::MQTTMessage, topic::MQTTTopic};
use protocol::placement_center::generate::mqtt::{
    CreateTopicRequest, DeleteTopicRequest, ListTopicRequest, SetTopicRetainMessageRequest,
};
use std::sync::Arc;

pub struct TopicStorage {
    client_poll: Arc<ClientPool>,
}

impl TopicStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return TopicStorage { client_poll };
    }

    pub async fn save_topic(&self, topic: MQTTTopic) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let request = CreateTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic.topic_name.clone(),
            content: topic.encode(),
        };
        match placement_create_topic(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    pub async fn delete_topic(&self, topic_name: String) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let request = DeleteTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
        };
        match placement_delete_topic(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    pub async fn topic_list(&self) -> Result<DashMap<String, MQTTTopic>, RobustMQError> {
        let config = broker_mqtt_conf();
        let request = ListTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: "".to_string(),
        };
        match placement_list_topic(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                let results = DashMap::with_capacity(2);
                for raw in reply.topics {
                    match serde_json::from_str::<MQTTTopic>(&raw) {
                        Ok(data) => {
                            results.insert(data.topic_name.clone(), data);
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }
                return Ok(results);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn get_topic(&self, topic_name: String) -> Result<Option<MQTTTopic>, RobustMQError> {
        let config = broker_mqtt_conf();
        let request = ListTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name,
        };
        match placement_list_topic(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                if reply.topics.len() == 0 {
                    return Ok(None);
                }
                let raw = reply.topics.get(0).unwrap();
                match serde_json::from_str::<MQTTTopic>(&raw) {
                    Ok(data) => return Ok(Some(data)),
                    Err(e) => {
                        return Err(RobustMQError::CommmonError(e.to_string()));
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn save_retain_message(
        &self,
        topic_name: String,
        retain_message: MQTTMessage,
    ) -> Result<(), RobustMQError> {
        let config = broker_mqtt_conf();
        let request = SetTopicRetainMessageRequest {
            cluster_name: config.cluster_name.clone(),
            topic_name: topic_name.clone(),
            retain_message: retain_message.encode(),
        };
        match placement_set_topic_retain_message(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }

    // Get the latest reserved message for the Topic dimension
    pub async fn get_retain_message(
        &self,
        topic_name: String,
    ) -> Result<Option<MQTTMessage>, RobustMQError> {
        let topic = match self.get_topic(topic_name).await {
            Ok(Some(data)) => data,
            Ok(None) => {
                return Err(RobustMQError::TopicDoesNotExist);
            }
            Err(e) => {
                return Err(e);
            }
        };

        if topic.retain_message.is_none() {
            return Ok(None);
        }
        let message =
            match serde_json::from_slice::<MQTTMessage>(topic.retain_message.unwrap().as_slice()) {
                Ok(data) => data,
                Err(e) => {
                    return Err(RobustMQError::CommmonError(e.to_string()));
                }
            };
        return Ok(Some(message));
    }
}
