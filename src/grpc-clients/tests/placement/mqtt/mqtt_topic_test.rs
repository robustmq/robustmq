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
    use common_base::tools::{now_second, unique_id};
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
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let client_id: String = unique_id().to_string();
        let topic_id: String = unique_id();
        let topic_name: String = "test_topic".to_string();
        let cluster_name: String = unique_id();
        let payload: String = "test_message".to_string();

        let mqtt_topic: MqttTopic = MqttTopic {
            topic_id: topic_id.clone(),
            topic_name: topic_name.clone(),
            cluster_name: cluster_name.clone(),
            retain_message: None,
            retain_message_expired_at: None,
            create_time: now_second(),
        };

        let request = CreateTopicRequest {
            cluster_name: cluster_name.clone(),
            topic_name: mqtt_topic.topic_name.clone(),
            content: mqtt_topic.encode(),
        };

        placement_create_topic(&client_pool, &addrs, request)
            .await
            .unwrap();

        contain_topic(
            cluster_name.clone(),
            topic_name.clone(),
            &client_pool,
            &addrs,
            mqtt_topic.clone(),
            true,
        )
        .await;

        let publish: Publish = Publish {
            dup: false,
            qos: qos(1).unwrap(),
            pkid: 0,
            retain: true,
            topic: Bytes::from(topic_name.clone()),
            payload: Bytes::from(payload.clone()),
        };
        let retain_message = MqttMessage::build_message(&client_id, &publish, &None, 600).encode();
        let retain_message_expired_at: u64 = 10000;

        // let mqtt_topic = MqttTopic {
        //     topic_id: topic_id.clone(),
        //     topic_name: topic_name.clone(),
        //     cluster_name: cluster_name.clone(),
        //     retain_message: Some(retain_message.clone()),
        //     retain_message_expired_at: Some(retain_message_expired_at),
        //     create_time: now_second(),
        // };

        let mut set_topic_retain_message_topic = mqtt_topic.clone();
        set_topic_retain_message_topic.retain_message = Some(retain_message.clone());
        set_topic_retain_message_topic.retain_message_expired_at = Some(retain_message_expired_at);

        let request = SetTopicRetainMessageRequest {
            cluster_name: cluster_name.clone(),
            topic_name: mqtt_topic.topic_name.clone(),
            retain_message: retain_message.clone(),
            retain_message_expired_at,
        };

        placement_set_topic_retain_message(&client_pool, &addrs, request)
            .await
            .unwrap();

        contain_topic(
            cluster_name.clone(),
            topic_name.clone(),
            &client_pool,
            &addrs,
            set_topic_retain_message_topic.clone(),
            true,
        )
        .await;

        let request = DeleteTopicRequest {
            cluster_name: cluster_name.clone(),
            topic_name: mqtt_topic.topic_name.clone(),
        };

        placement_delete_topic(&client_pool, &addrs, request)
            .await
            .unwrap();

        contain_topic(
            cluster_name.clone(),
            topic_name.clone(),
            &client_pool,
            &addrs,
            mqtt_topic.clone(),
            false,
        )
        .await;
    }

    async fn contain_topic(
        cluster_name: String,
        topic_name: String,
        client_pool: &ClientPool,
        addrs: &[impl AsRef<str>],
        mqtt_topic: MqttTopic,
        contain: bool,
    ) {
        let request = ListTopicRequest {
            cluster_name,
            topic_name,
        };
        let data = placement_list_topic(client_pool, addrs, request)
            .await
            .unwrap();

        let mut flag: bool = false;
        for raw in data.topics.clone() {
            let topic = serde_json::from_slice::<MqttTopic>(raw.as_slice()).unwrap();
            if topic == mqtt_topic {
                flag = true;
            }
        }
        if contain {
            assert!(!data.topics.is_empty());
            assert!(flag);
        } else {
            assert!(!flag);
            assert!(data.topics.is_empty());
        }
    }
}
