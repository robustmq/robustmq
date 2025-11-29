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
    use crate::common::get_placement_addr;
    use common_base::tools::{now_second, unique_id};
    use grpc_clients::meta::mqtt::call::{
        placement_create_topic, placement_delete_topic, placement_list_topic,
    };
    use grpc_clients::pool::ClientPool;
    use metadata_struct::mqtt::topic::MQTTTopic;
    use protocol::meta::meta_service_mqtt::{
        CreateTopicRequest, DeleteTopicRequest, ListTopicRequest,
    };
    use std::sync::Arc;

    #[tokio::test]

    async fn mqtt_topic_test() {
        let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(3));
        let addrs = vec![get_placement_addr()];
        let topic_name: String = unique_id();
        let cluster_name: String = unique_id();

        let mqtt_topic: MQTTTopic = MQTTTopic {
            topic_name: topic_name.clone(),
            create_time: now_second(),
        };

        let request = CreateTopicRequest {
            cluster_name: cluster_name.clone(),
            topic_name: mqtt_topic.topic_name.clone(),
            content: mqtt_topic.encode().unwrap(),
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
        mqtt_topic: MQTTTopic,
        contain: bool,
    ) {
        let request = ListTopicRequest {
            cluster_name,
            topic_name,
        };
        let mut data_stream = placement_list_topic(client_pool, addrs, request)
            .await
            .unwrap();

        let mut flag: bool = false;
        while let Some(data) = data_stream.message().await.unwrap() {
            let topic = MQTTTopic::decode(&data.topic).unwrap();
            if topic == mqtt_topic {
                flag = true;
            }
        }

        if contain {
            assert!(flag);
        } else {
            assert!(!flag);
        }
    }
}
