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
    use std::time::{Duration, Instant};

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
            publish_data,
        },
        ClientTestProperties,
    };
    use admin_server::mqtt::subscribe::{CreateAutoSubscribeReq, DeleteAutoSubscribeReq};
    use common_base::uuid::unique_id;
    use metadata_struct::tenant::DEFAULT_TENANT;
    use paho_mqtt::Message;
    use tokio::time::sleep;

    async fn wait_for_cache_sync() {
        sleep(Duration::from_secs(2)).await;
    }

    #[tokio::test]
    async fn sub_auto_basic_test() {
        let topic = format!("sub_auto_basic/{}", unique_id());
        let name = format!("rule_sub_auto_basic_{}", unique_id());
        let client_id = build_client_id("sub_auto_basic");
        let admin_client = create_test_env().await;

        admin_client
            .create_auto_subscribe(&CreateAutoSubscribeReq {
                name: name.clone(),
                desc: None,
                tenant: DEFAULT_TENANT.to_string(),
                topic: topic.clone(),
                qos: 1,
                no_local: false,
                retain_as_published: false,
                retained_handling: 0,
            })
            .await
            .unwrap();

        wait_for_cache_sync().await;

        let cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.clone(),
            addr: broker_addr_by_type("tcp"),
            ..Default::default()
        });
        let rx = cli.start_consuming();

        let message = "sub_auto_basic_test message".to_string();
        publish_data(&cli, Message::new(topic.clone(), message.clone(), 1), false);

        let start = Instant::now();
        let mut received = false;
        while start.elapsed() < Duration::from_secs(10) {
            if let Ok(Some(msg)) = rx.recv_timeout(Duration::from_secs(1)) {
                if msg.topic() == topic
                    && String::from_utf8(msg.payload().to_vec()).unwrap() == message
                {
                    received = true;
                    break;
                }
            }
        }
        assert!(received, "did not receive message on topic {topic}");

        distinct_conn(cli);

        admin_client
            .delete_auto_subscribe(&DeleteAutoSubscribeReq {
                tenant: DEFAULT_TENANT.to_string(),
                name: name.clone(),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn sub_auto_username_placeholder_test() {
        let uniq = unique_id();
        let topic_pattern = format!("device/${{username}}/cmd/{uniq}");
        let name = format!("rule_sub_auto_username_{uniq}");
        let client_id = build_client_id("sub_auto_username");
        let actual_topic = format!("device/admin/cmd/{uniq}");
        let admin_client = create_test_env().await;

        admin_client
            .create_auto_subscribe(&CreateAutoSubscribeReq {
                name: name.clone(),
                desc: None,
                tenant: DEFAULT_TENANT.to_string(),
                topic: topic_pattern.clone(),
                qos: 1,
                no_local: false,
                retain_as_published: false,
                retained_handling: 0,
            })
            .await
            .unwrap();

        wait_for_cache_sync().await;

        let cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.clone(),
            addr: broker_addr_by_type("tcp"),
            ..Default::default()
        });
        let rx = cli.start_consuming();

        let message = "sub_auto_username_placeholder_test message".to_string();
        publish_data(
            &cli,
            Message::new(actual_topic.clone(), message.clone(), 1),
            false,
        );

        let start = Instant::now();
        let mut received = false;
        while start.elapsed() < Duration::from_secs(10) {
            if let Ok(Some(msg)) = rx.recv_timeout(Duration::from_secs(1)) {
                if msg.topic() == actual_topic
                    && String::from_utf8(msg.payload().to_vec()).unwrap() == message
                {
                    received = true;
                    break;
                }
            }
        }
        assert!(received, "did not receive message on topic {actual_topic}");

        distinct_conn(cli);

        admin_client
            .delete_auto_subscribe(&DeleteAutoSubscribeReq {
                tenant: DEFAULT_TENANT.to_string(),
                name: name.clone(),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn sub_auto_host_placeholder_test() {
        let uniq = unique_id();
        let topic_pattern = format!("device/${{host}}/cmd/{uniq}");
        let name = format!("rule_sub_auto_host_{uniq}");
        let client_id = build_client_id("sub_auto_host");
        let actual_topic = format!("device/127.0.0.1/cmd/{uniq}");
        let admin_client = create_test_env().await;

        admin_client
            .create_auto_subscribe(&CreateAutoSubscribeReq {
                name: name.clone(),
                desc: None,
                tenant: DEFAULT_TENANT.to_string(),
                topic: topic_pattern.clone(),
                qos: 1,
                no_local: false,
                retain_as_published: false,
                retained_handling: 0,
            })
            .await
            .unwrap();

        wait_for_cache_sync().await;

        let cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.clone(),
            addr: broker_addr_by_type("tcp"),
            ..Default::default()
        });
        let rx = cli.start_consuming();

        let message = "sub_auto_host_placeholder_test message".to_string();
        publish_data(
            &cli,
            Message::new(actual_topic.clone(), message.clone(), 1),
            false,
        );

        let start = Instant::now();
        let mut received = false;
        while start.elapsed() < Duration::from_secs(10) {
            if let Ok(Some(msg)) = rx.recv_timeout(Duration::from_secs(1)) {
                if msg.topic() == actual_topic
                    && String::from_utf8(msg.payload().to_vec()).unwrap() == message
                {
                    received = true;
                    break;
                }
            }
        }
        assert!(received, "did not receive message on topic {actual_topic}");

        distinct_conn(cli);

        admin_client
            .delete_auto_subscribe(&DeleteAutoSubscribeReq {
                tenant: DEFAULT_TENANT.to_string(),
                name: name.clone(),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn sub_auto_clientid_placeholder_test() {
        let uniq = unique_id();
        let topic_pattern = format!("device/${{clientid}}/cmd/{uniq}");
        let name = format!("rule_sub_auto_clientid_{uniq}");
        let client_id = build_client_id("sub_auto_clientid");
        let actual_topic = format!("device/{client_id}/cmd/{uniq}");
        let admin_client = create_test_env().await;

        admin_client
            .create_auto_subscribe(&CreateAutoSubscribeReq {
                name: name.clone(),
                desc: None,
                tenant: DEFAULT_TENANT.to_string(),
                topic: topic_pattern.clone(),
                qos: 1,
                no_local: false,
                retain_as_published: false,
                retained_handling: 0,
            })
            .await
            .unwrap();

        wait_for_cache_sync().await;

        let cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.clone(),
            addr: broker_addr_by_type("tcp"),
            ..Default::default()
        });
        let rx = cli.start_consuming();

        let message = "sub_auto_clientid_placeholder_test message".to_string();
        publish_data(
            &cli,
            Message::new(actual_topic.clone(), message.clone(), 1),
            false,
        );

        let start = Instant::now();
        let mut received = false;
        while start.elapsed() < Duration::from_secs(10) {
            if let Ok(Some(msg)) = rx.recv_timeout(Duration::from_secs(1)) {
                if msg.topic() == actual_topic
                    && String::from_utf8(msg.payload().to_vec()).unwrap() == message
                {
                    received = true;
                    break;
                }
            }
        }
        assert!(received, "did not receive message on topic {actual_topic}");

        distinct_conn(cli);

        admin_client
            .delete_auto_subscribe(&DeleteAutoSubscribeReq {
                tenant: DEFAULT_TENANT.to_string(),
                name: name.clone(),
            })
            .await
            .unwrap();
    }
}
