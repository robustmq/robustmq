use clients::{
    placement::mqtt::call::{placement_create_topic, placement_delete_topic, placement_list_topic},
    poll::ClientPool,
};
use common_base::{config::broker_mqtt::broker_mqtt_conf, errors::RobustMQError};
use dashmap::DashMap;
use metadata_struct::mqtt::topic::MQTTTopic;
use protocol::placement_center::generate::mqtt::{
    CreateTopicRequest, DeleteTopicRequest, ListTopicRequest,
};
use std::sync::Arc;

pub struct TopicStorage {
    client_poll: Arc<ClientPool>,
}

impl TopicStorage {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        return TopicStorage { client_poll };
    }

    pub async fn save_topic(&self, topic_name: String) -> Result<String, RobustMQError> {
        let config = broker_mqtt_conf();
        let request = CreateTopicRequest {
            cluster_name: config.cluster_name.clone(),
            topic: Some(protocol::placement_center::generate::mqtt::Topic {
                topic_id: "".to_string(),
                topic_name: topic_name.clone(),
            }),
        };
        match placement_create_topic(
            self.client_poll.clone(),
            config.placement.server.clone(),
            request,
        )
        .await
        {
            Ok(reply) => {
                return Ok(reply.topic_id);
            }
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(format!(
                    "save user config error, error messsage:{}",
                    e.to_string()
                )))
            }
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
            Err(e) => {
                return Err(common_base::errors::RobustMQError::CommmonError(format!(
                    "save user config error, error messsage:{}",
                    e.to_string()
                )))
            }
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
                    results.insert(
                        raw.topic_name.clone(),
                        MQTTTopic {
                            topic_id: raw.topic_id.clone(),
                            topic_name: raw.topic_name.clone(),
                        },
                    );
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
                return Ok(Some(MQTTTopic {
                    topic_id: raw.topic_id.clone(),
                    topic_name: raw.topic_name.clone(),
                }));
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
