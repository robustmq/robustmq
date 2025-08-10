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
use crate::handler::error::MqttBrokerError;
use crate::observability::slow::slow_subscribe_data::SlowSubscribeData;
use crate::observability::slow::slow_subscribe_key::SlowSubscribeKey;
use crate::subscribe::common::Subscriber;
use std::collections::VecDeque;
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
    if check_capacity(metrics_cache_manager, config_num) {
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

fn check_capacity(metrics_cache_manager: &Arc<MetricsCacheManager>, config_num: usize) -> bool {
    // Check if the current number of slow subscribe records exceeds the configured limit
    metrics_cache_manager.slow_subscribe_info.len() > config_num + 1
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

pub fn read_slow_sub_record() -> Result<VecDeque<String>, MqttBrokerError> {
    todo!()
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
}
