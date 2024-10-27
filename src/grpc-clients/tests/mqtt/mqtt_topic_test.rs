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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;
    use grpc_clients::placement::mqtt::call::{
        placement_create_topic, placement_delete_topic, placement_list_topic,
        placement_set_topic_retain_message,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::message::MqttMessage;
    use metadata_struct::mqtt::topic::MqttTopic;
    use protocol::mqtt::common::{qos, Publish};
    use protocol::placement_center::placement_center_mqtt::{
        CreateTopicRequest, DeleteTopicRequest, ListTopicRequest, SetTopicRetainMessageRequest,
    };

    use crate::common::get_placement_addr;

    #[tokio::test]
    async fn mqtt_topic_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let client_id: String = "test_cient_id".to_string();
        let topic_id: String = "test_topic_ic".to_string();
        let topic_name: String = "test_topic".to_string();
        let cluster_name: String = "test_cluster".to_string();
        let payload: String = "test_message".to_string();
        let retain_message_expired_at: u64 = 10000;

        let mut mqtt_topic: MqttTopic = MqttTopic {
            topic_id: topic_id.clone(),
            topic_name: topic_name.clone(),
            retain_message: None,
            retain_message_expired_at: None,
        };

        let request = CreateTopicRequest {
            cluster_name: cluster_name.clone(),
            topic_name: mqtt_topic.topic_name.clone(),
            content: mqtt_topic.encode(),
        };
        match placement_create_topic(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let request = ListTopicRequest {
            cluster_name: cluster_name.clone(),
            topic_name: mqtt_topic.topic_name.clone(),
        };
        match placement_list_topic(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                assert!(!data.topics.is_empty());
                let mut flag: bool = false;
                for raw in data.topics {
                    let topic = serde_json::from_slice::<MqttTopic>(raw.as_slice()).unwrap();
                    if topic == mqtt_topic {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let publish: Publish = Publish {
            dup: false,
            qos: qos(1).unwrap(),
            pkid: 0,
            retain: true,
            topic: Bytes::from(topic_name.clone()),
            payload: Bytes::from(payload.clone()),
        };
        let retain_message = MqttMessage::build_message(&client_id, &publish, &None, 600);
        mqtt_topic = MqttTopic {
            topic_id: topic_id.clone(),
            topic_name: topic_name.clone(),
            retain_message: Some(retain_message.encode()),
            retain_message_expired_at: Some(retain_message_expired_at),
        };

        let request = SetTopicRetainMessageRequest {
            cluster_name: cluster_name.clone(),
            topic_name: mqtt_topic.topic_name.clone(),
            retain_message: mqtt_topic.retain_message.clone().unwrap(),
            retain_message_expired_at: mqtt_topic.retain_message_expired_at.unwrap(),
        };
        match placement_set_topic_retain_message(client_poll.clone(), addrs.clone(), request).await
        {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let request = ListTopicRequest {
            cluster_name: cluster_name.clone(),
            topic_name: mqtt_topic.topic_name.clone(),
        };
        match placement_list_topic(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                assert!(!data.topics.is_empty());
                let mut flag: bool = false;
                for raw in data.topics {
                    let topic = serde_json::from_slice::<MqttTopic>(raw.as_slice()).unwrap();
                    if topic == mqtt_topic {
                        flag = true;
                    }
                }
                assert!(flag);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let request = DeleteTopicRequest {
            cluster_name: cluster_name.clone(),
            topic_name: mqtt_topic.topic_name.clone(),
        };
        match placement_delete_topic(client_poll.clone(), addrs.clone(), request).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }

        let request = ListTopicRequest {
            cluster_name: cluster_name.clone(),
            topic_name: mqtt_topic.topic_name.clone(),
        };
        match placement_list_topic(client_poll.clone(), addrs.clone(), request).await {
            Ok(data) => {
                assert!(data.topics.is_empty());
                let mut flag: bool = false;
                for raw in data.topics {
                    let topic = serde_json::from_slice::<MqttTopic>(raw.as_slice()).unwrap();
                    if topic == mqtt_topic {
                        flag = true;
                    }
                }
                assert!(!flag);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}
