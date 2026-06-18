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
    use std::time::Instant;

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, create_test_env, distinct_conn,
            publish_data, subscribe_data_by_qos_with_timeout,
        },
        ClientTestProperties,
    };
    use admin_server::cluster::topic::{TopicDeleteRep, TopicListReq};
    use admin_server::mqtt::monitor::MonitorDataReq;
    use admin_server::mqtt::subscribe::{SubscribeListReq, SubscribeListRow};
    use admin_server::tool::PageReplyData;
    use common_base::uuid::unique_id;
    use metadata_struct::mqtt::topic::Topic;
    use paho_mqtt::{Message, MessageBuilder};
    use serde_json::Value;
    use tokio::time::{sleep, Duration};

    const SUBSCRIBE_TIMEOUT_SECS: u64 = 60;
    const TOPIC_CLEANUP_TIMEOUT_SECS: u64 = 30;
    const POLL_INTERVAL_SECS: u64 = 2;

    #[tokio::test]
    async fn topic_delete_integration_test() {
        let admin_client = create_test_env().await;
        let topic = format!("/topic_delete_test/{}", unique_id());
        let client_id = build_client_id("topic_delete_test");
        let message = "topic_delete_integration_test message".to_string();

        // ── Step 1: MQTT publish — generates the topic ────────────────────────
        let cli = connect_server(&ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.clone(),
            addr: broker_addr_by_type("tcp"),
            ..Default::default()
        });
        let msg = MessageBuilder::new()
            .payload(message.clone())
            .topic(topic.clone())
            .qos(1)
            .finalize();
        publish_data(&cli, msg, false);

        // ── Step 2: MQTT subscribe with explicit timeout ─────────────────────
        let call_fn = |msg: Message| String::from_utf8(msg.payload().to_vec()).unwrap() == message;
        assert!(
            subscribe_data_by_qos_with_timeout(
                &cli,
                &topic,
                1,
                Duration::from_secs(SUBSCRIBE_TIMEOUT_SECS),
                call_fn,
            )
            .is_ok(),
            "subscribe to topic '{topic}' timed out or failed"
        );

        // ── Step 3: Verify topic, subscription, and monitor data exist ───────
        let list_req = TopicListReq {
            topic_name: Some(topic.clone()),
            ..Default::default()
        };
        let topic_list: PageReplyData<Vec<Topic>> = admin_client
            .get_topic_list(&list_req)
            .await
            .expect("failed to query topic list before delete");
        assert!(
            !topic_list.data.is_empty(),
            "topic should exist after publish"
        );

        let sub_list: PageReplyData<Vec<SubscribeListRow>> = admin_client
            .get_subscribe_list(&SubscribeListReq {
                client_id: Some(client_id.clone()),
                ..Default::default()
            })
            .await
            .expect("failed to query subscribe list before delete");
        assert!(
            sub_list.total_count >= 1,
            "subscription should exist after subscribe"
        );

        // Verify monitor data returns non-empty data for topic counters
        let monitor_topic_resp: Value = admin_client
            .get_monitor_data(&MonitorDataReq {
                data_type: "topic_num".to_string(),
                topic_name: Some(topic.clone()),
                ..Default::default()
            })
            .await
            .expect("failed to query topic monitor data before delete");
        let topic_data_arr = monitor_topic_resp["data"]
            .as_array()
            .expect("topic monitor data should be a JSON array");
        assert!(
            !topic_data_arr.is_empty(),
            "topic monitor data should be non-empty before delete"
        );

        let monitor_sub_resp: Value = admin_client
            .get_monitor_data(&MonitorDataReq {
                data_type: "subscribe_num".to_string(),
                topic_name: Some(topic.clone()),
                ..Default::default()
            })
            .await
            .expect("failed to query subscription monitor data before delete");
        let sub_data_arr = monitor_sub_resp["data"]
            .as_array()
            .expect("subscription monitor data should be a JSON array");
        assert!(
            !sub_data_arr.is_empty(),
            "subscription monitor data should be non-empty before delete"
        );

        // ── Step 4: Delete the topic via admin API ────────────────────────────
        let delete_req = TopicDeleteRep {
            tenant: "public".to_string(),
            topic_name: topic.clone(),
        };
        admin_client
            .delete_topic(&delete_req)
            .await
            .expect("failed to delete topic");

        // ── Step 5: Poll until cleanup is complete, then verify everything is gone
        let start = Instant::now();
        let timeout = Duration::from_secs(TOPIC_CLEANUP_TIMEOUT_SECS);
        loop {
            let topic_list: PageReplyData<Vec<Topic>> = admin_client
                .get_topic_list(&list_req)
                .await
                .expect("failed to query topic list during cleanup poll");
            if topic_list.data.is_empty() {
                break;
            }
            if start.elapsed() >= timeout {
                panic!(
                    "topic '{}' still present after {} seconds of polling",
                    topic,
                    timeout.as_secs()
                );
            }
            sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }

        // Verify subscription is gone
        let sub_list_after: PageReplyData<Vec<SubscribeListRow>> = admin_client
            .get_subscribe_list(&SubscribeListReq {
                client_id: Some(client_id.clone()),
                ..Default::default()
            })
            .await
            .expect("failed to query subscribe list after delete");
        assert_eq!(
            sub_list_after.total_count, 0,
            "subscription should be removed after topic delete"
        );

        // Verify monitor data reports zero after deletion
        let monitor_topic_after: Value = admin_client
            .get_monitor_data(&MonitorDataReq {
                data_type: "topic_num".to_string(),
                topic_name: Some(topic.clone()),
                ..Default::default()
            })
            .await
            .expect("failed to query topic monitor data after delete");
        let data_arr = &monitor_topic_after["data"];
        assert!(
            data_arr.is_array() && data_arr.as_array().is_none_or(|a| a.is_empty()),
            "topic monitor data should be empty after delete, got: {monitor_topic_after}"
        );

        distinct_conn(cli);
    }
}
