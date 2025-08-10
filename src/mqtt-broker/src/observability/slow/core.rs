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

use crate::common::metrics_cache::MetricsCacheManager;
use crate::observability::slow::slow_subscribe_data::SlowSubscribeData;
use crate::observability::slow::slow_subscribe_key::SlowSubscribeKey;
use crate::subscribe::common::Subscriber;
use std::sync::Arc;

pub fn record_slow_subscribe_data(
    metrics_cache_manager: &Arc<MetricsCacheManager>,
    calculate_time: u64,
    config_num: usize,
    subscriber: &Subscriber,
) {
    let client_id = subscriber.client_id.clone();
    let topic_name = subscriber.topic_name.clone();
    let subscribe_name = subscriber.sub_path.clone();

    // Create new slow subscribe record
    let new_key = SlowSubscribeKey {
        time_span: calculate_time,
        client_id: client_id.clone(),
        topic_name: topic_name.clone(),
    };

    let new_data = SlowSubscribeData::build(
        subscribe_name,
        client_id.clone(),
        topic_name.clone(),
        calculate_time,
    );

    // Get existing index value if it exists
    let existing_index_value =
        metrics_cache_manager.get_slow_subscribe_index_value(client_id.clone(), topic_name.clone());

    // Handle existing record case
    if let Some(index_value) = existing_index_value {
        let existing_time_span = index_value.0;

        // Guard: Skip if new time is not greater than existing
        if calculate_time <= existing_time_span {
            return;
        }

        // Remove old record since new one has greater time_span
        let old_key = SlowSubscribeKey {
            time_span: existing_time_span,
            client_id: client_id.clone(),
            topic_name: topic_name.clone(),
        };
        metrics_cache_manager.remove_slow_subscribe_info(&old_key);

        // Insert new record and update index
        insert_new_record(
            metrics_cache_manager,
            new_key,
            new_data,
            client_id,
            topic_name,
            calculate_time,
        );
        return;
    }

    // Handle new record case (no existing index)
    handle_new_record_insertion(
        metrics_cache_manager,
        new_key,
        new_data,
        client_id,
        topic_name,
        calculate_time,
        config_num,
    );
}

/// Handle insertion of a new record when no existing index entry exists
fn handle_new_record_insertion(
    metrics_cache_manager: &Arc<MetricsCacheManager>,
    new_key: SlowSubscribeKey,
    new_data: SlowSubscribeData,
    client_id: String,
    topic_name: String,
    calculate_time: u64,
    config_num: usize,
) {
    // Check if we need to enforce capacity limit
    if !capacity_exceeded(metrics_cache_manager, config_num) {
        // No capacity limit reached, directly insert
        insert_new_record(
            metrics_cache_manager,
            new_key,
            new_data,
            client_id,
            topic_name,
            calculate_time,
        );
        return;
    }

    // Capacity limit reached, need to check if new record should replace existing minimum
    let min_key = metrics_cache_manager.slow_subscribe_info.min_key();

    // Guard: If no minimum key exists (edge case), insert directly
    let Some(min_key) = min_key else {
        insert_new_record(
            metrics_cache_manager,
            new_key,
            new_data,
            client_id,
            topic_name,
            calculate_time,
        );
        return;
    };

    // Guard: Skip if new time is not greater than minimum
    if calculate_time <= min_key.time_span {
        return;
    }

    // Remove minimum record and insert new one
    metrics_cache_manager
        .remove_slow_subscribe_index(min_key.client_id.clone(), min_key.topic_name.clone());
    metrics_cache_manager.remove_slow_subscribe_info(&min_key);

    insert_new_record(
        metrics_cache_manager,
        new_key,
        new_data,
        client_id,
        topic_name,
        calculate_time,
    );
}

fn capacity_exceeded(metrics_cache_manager: &Arc<MetricsCacheManager>, config_num: usize) -> bool {
    // Check if the current number of slow subscribe records exceeds the configured limit
    metrics_cache_manager.slow_subscribe_info.len() >= config_num
}

/// Insert new record and update index
fn insert_new_record(
    metrics_cache_manager: &Arc<MetricsCacheManager>,
    key: SlowSubscribeKey,
    data: SlowSubscribeData,
    client_id: String,
    topic_name: String,
    calculate_time: u64,
) {
    metrics_cache_manager.record_slow_subscribe_info(key, data);
    metrics_cache_manager.record_slow_subscribe_index(client_id, topic_name, calculate_time);
}

pub fn read_slow_sub_record(
    metrics_cache_manager: &Arc<MetricsCacheManager>,
) -> Vec<SlowSubscribeData> {
    // 从slow_subscribe_info中将所有的记录直接全部读取即可
    let mut slow_subscribe_data_list: Vec<SlowSubscribeData> = Vec::new();
    for (_, slow_subscribe_data) in metrics_cache_manager.slow_subscribe_info.iter_reverse() {
        slow_subscribe_data_list.push(slow_subscribe_data.clone());
    }
    slow_subscribe_data_list
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools::now_second;
    use protocol::mqtt::common::{MqttProtocol, QoS, RetainHandling};

    fn create_test_subscriber(client_id: &str, topic_name: &str, sub_path: &str) -> Subscriber {
        Subscriber {
            protocol: MqttProtocol::Mqtt4,
            client_id: client_id.to_string(),
            sub_path: sub_path.to_string(),
            rewrite_sub_path: None,
            topic_name: topic_name.to_string(),
            group_name: None,
            topic_id: "test_topic_id".to_string(),
            qos: QoS::AtMostOnce,
            nolocal: false,
            preserve_retain: false,
            retain_forward_rule: RetainHandling::OnEverySubscribe,
            subscription_identifier: None,
            create_time: now_second(),
        }
    }

    #[test]
    fn test_record_slow_subscribe_data_new_record() {
        // Test recording a new slow subscribe data when no existing record exists
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let subscriber = create_test_subscriber("client1", "test/topic", "test/topic");
        let calculate_time = 1000;
        let config_num = 10;

        record_slow_subscribe_data(
            &metrics_cache_manager,
            calculate_time,
            config_num,
            &subscriber,
        );

        // Verify the record was added
        let key = SlowSubscribeKey {
            time_span: calculate_time,
            client_id: "client1".to_string(),
            topic_name: "test/topic".to_string(),
        };

        assert!(metrics_cache_manager.slow_subscribe_info.contains_key(&key));
        assert_eq!(metrics_cache_manager.slow_subscribe_info.len(), 1);

        // Verify index was updated
        let index_value = metrics_cache_manager
            .get_slow_subscribe_index_value("client1".to_string(), "test/topic".to_string());
        assert!(index_value.is_some());
        assert_eq!(index_value.unwrap().0, calculate_time);
    }

    #[test]
    fn test_record_slow_subscribe_data_update_existing() {
        // Test updating an existing slow subscribe record with longer time_span
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let subscriber = create_test_subscriber("client1", "test/topic", "test/topic");

        // First record with shorter time_span
        let first_time = 500;
        record_slow_subscribe_data(&metrics_cache_manager, first_time, 10, &subscriber);

        // Second record with longer time_span (should replace the first one)
        let second_time = 1500;
        record_slow_subscribe_data(&metrics_cache_manager, second_time, 10, &subscriber);

        // Verify only the new record exists
        let old_key = SlowSubscribeKey {
            time_span: first_time,
            client_id: "client1".to_string(),
            topic_name: "test/topic".to_string(),
        };
        let new_key = SlowSubscribeKey {
            time_span: second_time,
            client_id: "client1".to_string(),
            topic_name: "test/topic".to_string(),
        };

        assert!(!metrics_cache_manager
            .slow_subscribe_info
            .contains_key(&old_key));
        assert!(metrics_cache_manager
            .slow_subscribe_info
            .contains_key(&new_key));
        assert_eq!(metrics_cache_manager.slow_subscribe_info.len(), 1);

        // Verify index was updated
        let index_value = metrics_cache_manager
            .get_slow_subscribe_index_value("client1".to_string(), "test/topic".to_string());
        assert!(index_value.is_some());
        assert_eq!(index_value.unwrap().0, second_time);
    }

    #[test]
    fn test_record_slow_subscribe_data_ignore_shorter_time() {
        // Test that shorter time_span records are ignored when existing record has longer time_span
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let subscriber = create_test_subscriber("client1", "test/topic", "test/topic");

        // First record with longer time_span
        let first_time = 2000;
        record_slow_subscribe_data(&metrics_cache_manager, first_time, 10, &subscriber);

        // Second record with shorter time_span (should be ignored)
        let second_time = 1000;
        record_slow_subscribe_data(&metrics_cache_manager, second_time, 10, &subscriber);

        // Verify only the first record exists
        let first_key = SlowSubscribeKey {
            time_span: first_time,
            client_id: "client1".to_string(),
            topic_name: "test/topic".to_string(),
        };
        let second_key = SlowSubscribeKey {
            time_span: second_time,
            client_id: "client1".to_string(),
            topic_name: "test/topic".to_string(),
        };

        assert!(metrics_cache_manager
            .slow_subscribe_info
            .contains_key(&first_key));
        assert!(!metrics_cache_manager
            .slow_subscribe_info
            .contains_key(&second_key));
        assert_eq!(metrics_cache_manager.slow_subscribe_info.len(), 1);

        // Verify index still points to the first record
        let index_value = metrics_cache_manager
            .get_slow_subscribe_index_value("client1".to_string(), "test/topic".to_string());
        assert!(index_value.is_some());
        assert_eq!(index_value.unwrap().0, first_time);
    }

    #[test]
    fn test_record_slow_subscribe_data_capacity_limit() {
        // Test that old records are removed when capacity limit is reached
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let config_num = 2; // Set small capacity for testing

        // Add first record
        let subscriber1 = create_test_subscriber("client1", "topic1", "topic1");
        record_slow_subscribe_data(&metrics_cache_manager, 1000, config_num, &subscriber1);

        // Add second record
        let subscriber2 = create_test_subscriber("client2", "topic2", "topic2");
        record_slow_subscribe_data(&metrics_cache_manager, 2000, config_num, &subscriber2);

        // Add third record with higher time_span (should remove the first record)
        let subscriber3 = create_test_subscriber("client3", "topic3", "topic3");
        record_slow_subscribe_data(&metrics_cache_manager, 3000, config_num, &subscriber3);

        // Verify capacity limit is respected
        assert_eq!(metrics_cache_manager.slow_subscribe_info.len(), config_num);

        // Verify the record with minimum time_span was removed
        let min_key = SlowSubscribeKey {
            time_span: 1000,
            client_id: "client1".to_string(),
            topic_name: "topic1".to_string(),
        };
        assert!(!metrics_cache_manager
            .slow_subscribe_info
            .contains_key(&min_key));

        // Verify the new record was added
        let new_key = SlowSubscribeKey {
            time_span: 3000,
            client_id: "client3".to_string(),
            topic_name: "topic3".to_string(),
        };
        assert!(metrics_cache_manager
            .slow_subscribe_info
            .contains_key(&new_key));
    }

    #[test]
    fn test_record_slow_subscribe_data_capacity_limit_ignore_smaller() {
        // Test that smaller time_span records are ignored when capacity is full
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let config_num = 1; // Set capacity to 1 for testing

        // Add first record
        let subscriber1 = create_test_subscriber("client1", "topic1", "topic1");
        record_slow_subscribe_data(&metrics_cache_manager, 2000, config_num, &subscriber1);

        // Try to add second record with smaller time_span (should be ignored)
        let subscriber2 = create_test_subscriber("client2", "topic2", "topic2");
        record_slow_subscribe_data(&metrics_cache_manager, 1000, config_num, &subscriber2);

        // Verify only the first record exists
        assert_eq!(metrics_cache_manager.slow_subscribe_info.len(), 1);

        let first_key = SlowSubscribeKey {
            time_span: 2000,
            client_id: "client1".to_string(),
            topic_name: "topic1".to_string(),
        };
        let second_key = SlowSubscribeKey {
            time_span: 1000,
            client_id: "client2".to_string(),
            topic_name: "topic2".to_string(),
        };

        assert!(metrics_cache_manager
            .slow_subscribe_info
            .contains_key(&first_key));
        assert!(!metrics_cache_manager
            .slow_subscribe_info
            .contains_key(&second_key));
    }

    #[test]
    fn test_record_slow_subscribe_data_multiple_clients() {
        // Test recording slow subscribe data for multiple different clients
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let config_num = 10;

        // Add records for different clients
        let clients = vec![
            ("client1", "topic1", 1000),
            ("client2", "topic2", 1500),
            ("client3", "topic3", 2000),
        ];

        for (client_id, topic_name, time_span) in &clients {
            let subscriber = create_test_subscriber(client_id, topic_name, topic_name);
            record_slow_subscribe_data(&metrics_cache_manager, *time_span, config_num, &subscriber);
        }

        // Verify all records were added
        assert_eq!(
            metrics_cache_manager.slow_subscribe_info.len(),
            clients.len()
        );

        for (client_id, topic_name, time_span) in clients {
            let key = SlowSubscribeKey {
                time_span,
                client_id: client_id.to_string(),
                topic_name: topic_name.to_string(),
            };
            assert!(metrics_cache_manager.slow_subscribe_info.contains_key(&key));

            // Verify index
            let index_value = metrics_cache_manager
                .get_slow_subscribe_index_value(client_id.to_string(), topic_name.to_string());
            assert!(index_value.is_some());
            assert_eq!(index_value.unwrap().0, time_span);
        }
    }

    #[test]
    fn test_read_slow_sub_record_empty() {
        // Test reading from empty cache
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());

        let result = read_slow_sub_record(&metrics_cache_manager);

        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_read_slow_sub_record_single_record() {
        // Test reading a single slow subscribe record
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let subscriber = create_test_subscriber("client1", "test/topic", "test/topic");
        let calculate_time = 1500;

        // Insert one record
        record_slow_subscribe_data(&metrics_cache_manager, calculate_time, 10, &subscriber);

        let result = read_slow_sub_record(&metrics_cache_manager);

        assert_eq!(result.len(), 1);
        let record = &result[0];
        assert_eq!(record.client_id, "client1");
        assert_eq!(record.topic_name, "test/topic");
        assert_eq!(record.subscribe_name, "test/topic");
        assert_eq!(record.time_span, calculate_time);
    }

    #[test]
    fn test_read_slow_sub_record_multiple_records() {
        // Test reading multiple slow subscribe records
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let config_num = 10;

        // Insert multiple records with different time_spans
        let test_data = vec![
            ("client1", "topic1", 1000),
            ("client2", "topic2", 2000),
            ("client3", "topic3", 1500),
        ];

        for (client_id, topic_name, time_span) in &test_data {
            let subscriber = create_test_subscriber(client_id, topic_name, topic_name);
            record_slow_subscribe_data(&metrics_cache_manager, *time_span, config_num, &subscriber);
        }

        let result = read_slow_sub_record(&metrics_cache_manager);

        assert_eq!(result.len(), test_data.len());

        // Verify all records are present (order might be different due to reverse iteration)
        for (expected_client_id, expected_topic_name, expected_time_span) in test_data {
            let found = result.iter().find(|record| {
                record.client_id == expected_client_id
                    && record.topic_name == expected_topic_name
                    && record.time_span == expected_time_span
            });
            assert!(
                found.is_some(),
                "Expected record not found: client_id={expected_client_id}, topic_name={expected_topic_name}, time_span={expected_time_span}"
            );
        }
    }

    #[test]
    fn test_read_slow_sub_record_reverse_order() {
        // Test that records are returned in reverse order (highest time_span first)
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let config_num = 10;

        // Insert records with ascending time_spans
        let time_spans = [1000, 2000, 3000, 4000];
        for (i, &time_span) in time_spans.iter().enumerate() {
            let client_id = format!("client{}", i + 1);
            let topic_name = format!("topic{}", i + 1);
            let subscriber = create_test_subscriber(&client_id, &topic_name, &topic_name);
            record_slow_subscribe_data(&metrics_cache_manager, time_span, config_num, &subscriber);
        }

        let result = read_slow_sub_record(&metrics_cache_manager);

        assert_eq!(result.len(), time_spans.len());

        // Verify records are in descending order of time_span (reverse iteration)
        let mut prev_time_span = u64::MAX;
        for record in result {
            assert!(
                record.time_span <= prev_time_span,
                "Records should be in descending order of time_span"
            );
            prev_time_span = record.time_span;
        }
    }

    #[test]
    fn test_read_slow_sub_record_after_updates() {
        // Test reading records after some updates (old records removed)
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let subscriber = create_test_subscriber("client1", "test/topic", "test/topic");

        // Insert initial record
        record_slow_subscribe_data(&metrics_cache_manager, 1000, 10, &subscriber);

        // Update with higher time_span (should replace the old one)
        record_slow_subscribe_data(&metrics_cache_manager, 2500, 10, &subscriber);

        let result = read_slow_sub_record(&metrics_cache_manager);

        assert_eq!(result.len(), 1);
        let record = &result[0];
        assert_eq!(record.client_id, "client1");
        assert_eq!(record.topic_name, "test/topic");
        assert_eq!(record.time_span, 2500); // Should be the updated time_span
    }

    #[test]
    fn test_read_slow_sub_record_with_capacity_limit() {
        // Test reading records when capacity limit has been enforced
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let config_num = 2; // Small capacity limit

        // Insert more records than capacity allows
        let test_data = vec![
            ("client1", "topic1", 1000),
            ("client2", "topic2", 2000),
            ("client3", "topic3", 3000), // This should remove client1 record
        ];

        for (client_id, topic_name, time_span) in &test_data {
            let subscriber = create_test_subscriber(client_id, topic_name, topic_name);
            record_slow_subscribe_data(&metrics_cache_manager, *time_span, config_num, &subscriber);
        }

        let result = read_slow_sub_record(&metrics_cache_manager);

        // Should only have config_num records
        assert_eq!(result.len(), config_num);

        // Should not contain the record with minimum time_span (client1)
        let has_client1 = result.iter().any(|record| record.client_id == "client1");
        assert!(
            !has_client1,
            "client1 record should have been removed due to capacity limit"
        );

        // Should contain the records with higher time_spans
        let has_client2 = result.iter().any(|record| record.client_id == "client2");
        let has_client3 = result.iter().any(|record| record.client_id == "client3");
        assert!(has_client2, "client2 record should be present");
        assert!(has_client3, "client3 record should be present");
    }

    #[test]
    fn test_read_slow_sub_record_data_integrity() {
        // Test that all fields of SlowSubscribeData are correctly preserved
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let subscriber =
            create_test_subscriber("test_client", "test/topic/path", "custom/sub/path");
        let calculate_time = 5000;

        record_slow_subscribe_data(&metrics_cache_manager, calculate_time, 10, &subscriber);

        let result = read_slow_sub_record(&metrics_cache_manager);

        assert_eq!(result.len(), 1);
        let record = &result[0];

        // Verify all fields
        assert_eq!(record.client_id, "test_client");
        assert_eq!(record.topic_name, "test/topic/path");
        assert_eq!(record.subscribe_name, "custom/sub/path");
        assert_eq!(record.time_span, calculate_time);
        assert!(
            !record.node_info.is_empty(),
            "node_info should not be empty"
        );
        assert!(record.create_time > 0, "create_time should be set");
    }

    #[test]
    fn test_read_slow_sub_record_concurrent_modification() {
        // Test reading while records might be modified (basic test)
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());

        // Add some initial records
        for i in 1..=3 {
            let client_id = format!("client{i}");
            let topic_name = format!("topic{i}");
            let subscriber = create_test_subscriber(&client_id, &topic_name, &topic_name);
            record_slow_subscribe_data(&metrics_cache_manager, (i * 1000) as u64, 10, &subscriber);
        }

        // Read records
        let result1 = read_slow_sub_record(&metrics_cache_manager);
        assert_eq!(result1.len(), 3);

        // Add another record
        let subscriber = create_test_subscriber("client4", "topic4", "topic4");
        record_slow_subscribe_data(&metrics_cache_manager, 4000, 10, &subscriber);

        // Read again
        let result2 = read_slow_sub_record(&metrics_cache_manager);
        assert_eq!(result2.len(), 4);
    }

    #[test]
    fn test_read_slow_sub_record_reverse_order_after_updates() {
        // Test that records maintain reverse order after updates
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let config_num = 10;

        // Step 1: Insert records with ascending time_spans
        let initial_time_spans = [1000, 2000, 3000, 4000];
        for (i, &time_span) in initial_time_spans.iter().enumerate() {
            let client_id = format!("client{}", i + 1);
            let topic_name = format!("topic{}", i + 1);
            let subscriber = create_test_subscriber(&client_id, &topic_name, &topic_name);
            record_slow_subscribe_data(&metrics_cache_manager, time_span, config_num, &subscriber);
        }

        // Step 2: First read - verify initial reverse order (descending: 4000, 3000, 2000, 1000)
        let result_before = read_slow_sub_record(&metrics_cache_manager);
        assert_eq!(result_before.len(), initial_time_spans.len());

        // Verify descending order
        let expected_initial_order = [4000, 3000, 2000, 1000];
        for (i, record) in result_before.iter().enumerate() {
            assert_eq!(
                record.time_span, expected_initial_order[i],
                "Initial order incorrect at position {}: expected {}, got {}",
                i, expected_initial_order[i], record.time_span
            );
        }

        // Step 3: Update client2's record with the highest time_span (should move to first position)
        let subscriber2_updated = create_test_subscriber("client2", "topic2", "topic2");
        let new_highest_time = 5500;
        record_slow_subscribe_data(
            &metrics_cache_manager,
            new_highest_time,
            config_num,
            &subscriber2_updated,
        );

        // Step 4: Second read - verify new order after first update
        let result_after_first_update = read_slow_sub_record(&metrics_cache_manager);
        assert_eq!(result_after_first_update.len(), initial_time_spans.len()); // Same count

        // New expected order: 5500(client2), 4000(client4), 3000(client3), 1000(client1)
        let expected_after_first = [5500, 4000, 3000, 1000];
        for (i, record) in result_after_first_update.iter().enumerate() {
            assert_eq!(
                record.time_span, expected_after_first[i],
                "Order after first update incorrect at position {}: expected {}, got {}",
                i, expected_after_first[i], record.time_span
            );
        }

        // Verify client2 is now at the top
        assert_eq!(
            result_after_first_update[0].client_id, "client2",
            "client2 should be first after update"
        );
        assert_eq!(result_after_first_update[0].time_span, new_highest_time);

        // Step 5: Update client1's record with medium time_span (should move to middle position)
        let subscriber1_updated = create_test_subscriber("client1", "topic1", "topic1");
        let medium_time = 3500;
        record_slow_subscribe_data(
            &metrics_cache_manager,
            medium_time,
            config_num,
            &subscriber1_updated,
        );

        // Step 6: Third read - verify final order after second update
        let result_after_second_update = read_slow_sub_record(&metrics_cache_manager);
        assert_eq!(result_after_second_update.len(), initial_time_spans.len()); // Same count

        // Final expected order: 5500(client2), 4000(client4), 3500(client1), 3000(client3)
        let expected_final_order = [5500, 4000, 3500, 3000];
        for (i, record) in result_after_second_update.iter().enumerate() {
            assert_eq!(
                record.time_span, expected_final_order[i],
                "Final order incorrect at position {}: expected {}, got {}",
                i, expected_final_order[i], record.time_span
            );
        }

        // Verify client1 is now in the correct middle position
        let client1_record = result_after_second_update
            .iter()
            .find(|r| r.client_id == "client1")
            .unwrap();
        assert_eq!(client1_record.time_span, medium_time);

        // Step 7: Verify all records maintain reverse order throughout
        let mut prev_time_span = u64::MAX;
        for record in &result_after_second_update {
            assert!(
                record.time_span <= prev_time_span,
                "Records should be in descending order of time_span, but {} > {}",
                record.time_span,
                prev_time_span
            );
            prev_time_span = record.time_span;
        }

        // Step 8: Verify no old records remain
        let has_old_client2_2000 = result_after_second_update
            .iter()
            .any(|r| r.client_id == "client2" && r.time_span == 2000);
        let has_old_client1_1000 = result_after_second_update
            .iter()
            .any(|r| r.client_id == "client1" && r.time_span == 1000);

        assert!(
            !has_old_client2_2000,
            "Old client2 record (2000) should be gone"
        );
        assert!(
            !has_old_client1_1000,
            "Old client1 record (1000) should be gone"
        );
    }
}
