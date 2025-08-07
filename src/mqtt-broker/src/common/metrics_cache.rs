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

use crate::common::concurrent_btree_map::ShardedConcurrentBTreeMap;
use crate::common::types::ResultMqttBrokerError;
use crate::observability::metrics::server::{
    record_broker_connections_max, record_broker_connections_num,
};
use crate::observability::slow::slow_subscribe_key::SlowSubscribeKey;
use crate::observability::slow::sub::SlowSubscribeData;
use crate::{
    common::tool::loop_select, handler::cache::CacheManager,
    server::common::connection_manager::ConnectionManager, subscribe::manager::SubscribeManager,
};
use common_base::tools::now_second;
use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tracing::info;

#[derive(Default, Clone)]
pub struct MetricsCacheManager {
    pub connection_num: DashMap<u64, u32>,
    pub topic_num: DashMap<u64, u32>,
    pub subscribe_num: DashMap<u64, u32>,
    pub message_in_num: DashMap<u64, u32>,
    pub message_out_num: DashMap<u64, u32>,
    pub message_drop_num: DashMap<u64, u32>,
    pub slow_subscribe_index: DashMap<(String, String), (u64, String, String)>,
    pub slow_subscribe_info: ShardedConcurrentBTreeMap<SlowSubscribeKey, SlowSubscribeData>,
}

impl MetricsCacheManager {
    pub fn new() -> Self {
        MetricsCacheManager {
            connection_num: DashMap::with_capacity(4),
            topic_num: DashMap::with_capacity(4),
            subscribe_num: DashMap::with_capacity(4),
            message_in_num: DashMap::with_capacity(4),
            message_out_num: DashMap::with_capacity(4),
            message_drop_num: DashMap::with_capacity(4),
            slow_subscribe_index: DashMap::new(),
            slow_subscribe_info: ShardedConcurrentBTreeMap::new(),
        }
    }

    pub fn get_slow_subscribe_index_value(
        &self,
        client_id: String,
        topic_name: String,
    ) -> Option<(u64, String, String)> {
        // Return the latest slow subscribe data for the given client_id and topic_name
        self.slow_subscribe_index
            .get(&(client_id, topic_name))
            .map(|entry| entry.value().clone())
    }

    pub fn record_slow_subscribe_index(
        &self,
        client_id: String,
        topic_name: String,
        time_span: u64,
    ) {
        self.slow_subscribe_index.insert(
            (client_id.clone(), topic_name.clone()),
            (time_span, client_id, topic_name),
        );
    }

    pub fn get_slow_subscribe_info(
        &self,
        slow_subscribe_key: SlowSubscribeKey,
    ) -> Option<SlowSubscribeData> {
        self.slow_subscribe_info.get(&slow_subscribe_key)
    }

    pub fn record_slow_subscribe_info(
        &self,
        slow_subscribe_key: SlowSubscribeKey,
        slow_subscribe_data: SlowSubscribeData,
    ) {
        self.slow_subscribe_info
            .insert(slow_subscribe_key, slow_subscribe_data);
    }

    /// Removes a slow subscribe index entry by client_id and topic_name
    /// Returns the removed value if it existed
    pub fn remove_slow_subscribe_index(
        &self,
        client_id: String,
        topic_name: String,
    ) -> Option<(u64, String, String)> {
        self.slow_subscribe_index
            .remove(&(client_id, topic_name))
            .map(|(_, value)| value)
    }

    /// Removes a slow subscribe info entry by key
    /// Returns the removed value if it existed
    pub fn remove_slow_subscribe_info(
        &self,
        slow_subscribe_key: &SlowSubscribeKey,
    ) -> Option<SlowSubscribeData> {
        self.slow_subscribe_info.remove(slow_subscribe_key)
    }

    pub fn record_connection_num(&self, time: u64, num: u32) {
        self.connection_num.insert(time, num);
    }

    pub fn record_topic_num(&self, time: u64, num: u32) {
        self.topic_num.insert(time, num);
    }

    pub fn record_subscribe_num(&self, time: u64, num: u32) {
        self.subscribe_num.insert(time, num);
    }

    pub fn record_message_in_num(&self, time: u64, num: u32) {
        self.message_in_num.insert(time, num);
    }

    pub fn record_message_out_num(&self, time: u64, num: u32) {
        self.message_out_num.insert(time, num);
    }

    pub fn record_message_drop_num(&self, time: u64, num: u32) {
        self.message_drop_num.insert(time, num);
    }

    pub fn get_connection_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.connection_num.clone(), start_time, end_time)
    }

    pub fn get_topic_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.topic_num.clone(), start_time, end_time)
    }

    pub fn get_subscribe_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.subscribe_num.clone(), start_time, end_time)
    }

    pub fn get_message_in_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.message_in_num.clone(), start_time, end_time)
    }

    pub fn get_message_out_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.message_out_num.clone(), start_time, end_time)
    }

    pub fn get_message_drop_num_by_time(
        &self,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        self.search_by_time(self.message_drop_num.clone(), start_time, end_time)
    }

    pub fn export_metrics(&self) {
        if let Some(connection_num) = self.latest_by_time(&self.connection_num) {
            record_broker_connections_num(connection_num as i64);
            record_broker_connections_max(connection_num as i64);
        }
    }

    // Get the value within a given time interval
    fn search_by_time(
        &self,
        data_list: DashMap<u64, u32>,
        start_time: u64,
        end_time: u64,
    ) -> Vec<HashMap<String, u64>> {
        let mut results = Vec::new();
        for (time, value) in data_list {
            if time >= start_time && time <= end_time {
                let mut raw = HashMap::new();
                raw.insert("date".to_string(), time);
                raw.insert("value".to_string(), value as u64);
                results.push(raw);
            }
        }
        results
    }

    // Get the latest time value
    fn latest_by_time(&self, data_list: &DashMap<u64, u32>) -> Option<u32> {
        data_list
            .iter()
            .max_by_key(|kv| *kv.key())
            .map(|kv| *kv.value())
    }
}

pub fn metrics_record_thread(
    metrics_cache_manager: Arc<MetricsCacheManager>,
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    time_window: u64,
    stop_send: broadcast::Sender<bool>,
) {
    info!("Metrics record thread start successfully");
    tokio::spawn(async move {
        let record_func = async || -> ResultMqttBrokerError {
            let now = now_second();
            let metrics_cache_manager = metrics_cache_manager.clone();
            let connection_manager = connection_manager.clone();
            metrics_cache_manager
                .record_connection_num(now, connection_manager.connections.len() as u32);
            metrics_cache_manager.record_topic_num(now, cache_manager.topic_info.len() as u32);
            metrics_cache_manager
                .record_subscribe_num(now, subscribe_manager.subscribe_list.len() as u32);
            metrics_cache_manager.record_message_in_num(now, 1000);
            metrics_cache_manager.record_message_out_num(now, 1000);
            metrics_cache_manager.record_message_drop_num(now, 30);

            // Many system metrics can be reused here. We only need to get the instantaneous value.
            // However, it should be noted that prometheus export itself is periodic,
            // and the current function is also periodic.
            // Further, we can conclude that the time range of
            // indicator export is [min(metrics_export_interval,time_window), metrics_export_interval + time_window]
            metrics_cache_manager.export_metrics();
            Ok(())
        };
        loop_select(record_func, time_window, &stop_send).await;
    });
}

pub fn metrics_gc_thread(
    metrics_cache_manager: Arc<MetricsCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    info!("Metrics gc thread start successfully");
    tokio::spawn(async move {
        let record_func = async || -> ResultMqttBrokerError {
            let now_time = now_second();
            let save_time = 3600 * 24 * 3;

            // connection_num
            for (time, _) in metrics_cache_manager.connection_num.clone() {
                if (time + save_time) < now_time {
                    metrics_cache_manager.connection_num.remove(&time);
                }
            }

            // topic_num
            for (time, _) in metrics_cache_manager.topic_num.clone() {
                if (time + save_time) < now_time {
                    metrics_cache_manager.topic_num.remove(&time);
                }
            }

            // subscribe_num
            for (time, _) in metrics_cache_manager.subscribe_num.clone() {
                if (time + save_time) < now_time {
                    metrics_cache_manager.subscribe_num.remove(&time);
                }
            }

            // message_in_num
            for (time, _) in metrics_cache_manager.message_in_num.clone() {
                if (time + save_time) < now_time {
                    metrics_cache_manager.message_in_num.remove(&time);
                }
            }

            // message_out_num
            for (time, _) in metrics_cache_manager.message_out_num.clone() {
                if (time + save_time) < now_time {
                    metrics_cache_manager.message_out_num.remove(&time);
                }
            }

            // message_drop_num
            for (time, _) in metrics_cache_manager.message_drop_num.clone() {
                if (time + save_time) < now_time {
                    metrics_cache_manager.message_drop_num.remove(&time);
                }
            }

            Ok(())
        };
        loop_select(record_func, 3600, &stop_send).await;
    });
}

#[cfg(test)]
mod test {
    use crate::observability::slow::slow_subscribe_key::SlowSubscribeKey;
    use crate::observability::slow::sub::SlowSubscribeData;
    use std::{
        net::{Ipv4Addr, SocketAddrV4},
        sync::Arc,
        time::Duration,
    };

    use common_base::tools::now_second;
    use grpc_clients::pool::ClientPool;
    use tokio::{sync::broadcast, time::sleep};

    use crate::{
        common::metrics_cache::{metrics_gc_thread, metrics_record_thread, MetricsCacheManager},
        handler::cache::CacheManager,
        server::common::{
            connection::{NetworkConnection, NetworkConnectionType},
            connection_manager::ConnectionManager,
        },
        subscribe::manager::SubscribeManager,
    };

    #[tokio::test]
    pub async fn minute_test() {
        let mut times = 0;
        loop {
            if times >= 1 {
                break;
            }
            let now = now_second();
            if now % 60 == 0 {
                println!("{now}");
                times += 1;
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    #[tokio::test]
    pub async fn metrics_cache_test() {
        let metrics_cache_manager = Arc::new(MetricsCacheManager::new());
        let (stop_send, _) = broadcast::channel(2);
        let client_pool = Arc::new(ClientPool::new(100));
        let cluster_name = "test".to_string();
        let cache_manager = Arc::new(CacheManager::new(client_pool, cluster_name));
        let subscribe_manager = Arc::new(SubscribeManager::new());

        // add mock connection
        let connection_mgr = ConnectionManager::new(cache_manager.clone());
        connection_mgr.add_connection(NetworkConnection::new(
            NetworkConnectionType::Tls,
            std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)),
            None,
        ));
        let connection_manager = Arc::new(connection_mgr);

        let now = now_second();
        metrics_gc_thread(metrics_cache_manager.clone(), stop_send.clone());
        metrics_record_thread(
            metrics_cache_manager.clone(),
            cache_manager,
            subscribe_manager,
            connection_manager,
            1,
            stop_send,
        );

        sleep(Duration::from_secs(10)).await;
        assert_eq!(metrics_cache_manager.connection_num.len(), 10);
        assert_eq!(metrics_cache_manager.topic_num.len(), 10);
        assert_eq!(metrics_cache_manager.subscribe_num.len(), 10);
        assert_eq!(metrics_cache_manager.message_in_num.len(), 10);
        assert_eq!(metrics_cache_manager.message_out_num.len(), 10);
        assert_eq!(metrics_cache_manager.message_drop_num.len(), 10);

        let start_time = now + 2;
        let end_time = now + 8;
        assert_eq!(
            metrics_cache_manager
                .get_connection_num_by_time(start_time, end_time)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .get_topic_num_by_time(start_time, end_time)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .get_subscribe_num_by_time(start_time, end_time)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .get_message_in_num_by_time(start_time, end_time)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .get_message_out_num_by_time(start_time, end_time)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .get_message_drop_num_by_time(start_time, end_time)
                .len(),
            7
        );

        assert_eq!(
            metrics_cache_manager.latest_by_time(&metrics_cache_manager.connection_num),
            Some(1)
        )
    }

    #[test]
    fn test_slow_subscribe_index_insert_and_get() {
        let manager = MetricsCacheManager::new();

        // Insert a value
        manager.record_slow_subscribe_index("client1".into(), "topicA".into(), 10);

        // Get the value
        let result = manager.get_slow_subscribe_index_value("client1".into(), "topicA".into());
        assert_eq!(result, Some((10, "client1".into(), "topicA".into())));
    }

    #[test]
    fn test_slow_subscribe_index_update_and_get() {
        let manager = MetricsCacheManager::new();

        // Insert a value
        manager.record_slow_subscribe_index("client1".into(), "topicA".into(), 10);

        // Update the value
        manager.record_slow_subscribe_index("client1".into(), "topicA".into(), 20);

        // Get the updated value
        let result = manager.get_slow_subscribe_index_value("client1".into(), "topicA".into());
        assert_eq!(result, Some((20, "client1".into(), "topicA".into())));
    }

    #[test]
    fn test_slow_subscribe_index_get_nonexistent() {
        let manager = MetricsCacheManager::new();

        // Get a nonexistent value
        let result = manager.get_slow_subscribe_index_value("client1".into(), "topicA".into());
        assert_eq!(result, None);
    }

    #[test]
    fn test_slow_subscribe_info_insert_and_get() {
        let manager = MetricsCacheManager::new();

        // Insert a value
        let key = SlowSubscribeKey {
            time_span: 10,
            client_id: "client1".into(),
            topic_name: "topicA".into(),
        };
        let data = SlowSubscribeData {
            subscribe_name: "subscribe_name".to_string(),
            client_id: "client_id".to_string(),
            topic_name: "topic_name".to_string(),
            node_info: "node_info".to_string(),
            time_span: 0,
            create_time: 0,
        };
        manager.record_slow_subscribe_info(key.clone(), data.clone());

        // Get the value
        let result = manager.get_slow_subscribe_info(key);
        assert_eq!(result, Some(data));
    }

    #[test]
    fn test_slow_subscribe_info_update_and_get() {
        let manager = MetricsCacheManager::new();

        // Insert a value
        let key = SlowSubscribeKey {
            time_span: 10,
            client_id: "client1".into(),
            topic_name: "topicA".into(),
        };
        let data1 = SlowSubscribeData {
            subscribe_name: "subscribe_name".to_string(),
            client_id: "client_id".to_string(),
            topic_name: "topic_name".to_string(),
            node_info: "node_info".to_string(),
            time_span: 0,
            create_time: 0,
        };
        manager.record_slow_subscribe_info(key.clone(), data1);

        // Update the value
        let data2 = SlowSubscribeData {
            subscribe_name: "subscribe_name_1".to_string(),
            client_id: "client_id_1".to_string(),
            topic_name: "topic_name_1".to_string(),
            node_info: "node_info_1".to_string(),
            time_span: 0,
            create_time: 0,
        };
        manager.record_slow_subscribe_info(key.clone(), data2.clone());

        // Get the updated value
        let result = manager.get_slow_subscribe_info(key);
        assert_eq!(result, Some(data2));
    }

    #[test]
    fn test_slow_subscribe_info_get_nonexistent() {
        let manager = MetricsCacheManager::new();

        // Get a nonexistent value
        let key = SlowSubscribeKey {
            time_span: 10,
            client_id: "client1".into(),
            topic_name: "topicA".into(),
        };
        let result = manager.get_slow_subscribe_info(key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_slow_subscribe_info_insert_and_iterate() {
        let manager = MetricsCacheManager::new();

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_name_1".to_string(),
                client_id: "client_id".to_string(),
                topic_name: "topic_name".to_string(),
                node_info: "node_info".to_string(),
                time_span: 0,
                create_time: 0,
            },
        );

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 15,
                client_id: "client2".into(),
                topic_name: "topicB".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_name_2".to_string(),
                client_id: "client_id".to_string(),
                topic_name: "topic_name".to_string(),
                node_info: "node_info".to_string(),
                time_span: 0,
                create_time: 0,
            },
        );

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicC".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_name_3".to_string(),
                client_id: "client_id".to_string(),
                topic_name: "topic_name".to_string(),
                node_info: "node_info".to_string(),
                time_span: 0,
                create_time: 0,
            },
        );

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 5,
                client_id: "client3".into(),
                topic_name: "topicD".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_name_4".to_string(),
                client_id: "client_id".to_string(),
                topic_name: "topic_name".to_string(),
                node_info: "node_info".to_string(),
                time_span: 0,
                create_time: 0,
            },
        );

        let expected_order = vec![
            SlowSubscribeKey {
                time_span: 5,
                client_id: "client3".into(),
                topic_name: "topicD".into(),
            },
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into(),
            },
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicC".into(),
            },
            SlowSubscribeKey {
                time_span: 15,
                client_id: "client2".into(),
                topic_name: "topicB".into(),
            },
        ];

        // Use the keys() method from ConcurrentBTreeMap which returns keys in sorted order
        let actual_order = manager.slow_subscribe_info.keys();

        assert_eq!(actual_order, expected_order);
    }
    #[test]
    fn test_slow_subscribe_info_insert_and_iterate_reverse() {
        let manager = MetricsCacheManager::new();

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_name_1".to_string(),
                client_id: "client_id".to_string(),
                topic_name: "topic_name".to_string(),
                node_info: "node_info".to_string(),
                time_span: 0,
                create_time: 0,
            },
        );

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 15,
                client_id: "client2".into(),
                topic_name: "topicB".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_name_2".to_string(),
                client_id: "client_id".to_string(),
                topic_name: "topic_name".to_string(),
                node_info: "node_info".to_string(),
                time_span: 0,
                create_time: 0,
            },
        );

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicC".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_name_3".to_string(),
                client_id: "client_id".to_string(),
                topic_name: "topic_name".to_string(),
                node_info: "node_info".to_string(),
                time_span: 0,
                create_time: 0,
            },
        );

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 5,
                client_id: "client3".into(),
                topic_name: "topicD".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_name_4".to_string(),
                client_id: "client_id".to_string(),
                topic_name: "topic_name".to_string(),
                node_info: "node_info".to_string(),
                time_span: 0,
                create_time: 0,
            },
        );

        let expected_order = vec![
            SlowSubscribeKey {
                time_span: 15,
                client_id: "client2".into(),
                topic_name: "topicB".into(),
            },
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicC".into(),
            },
            SlowSubscribeKey {
                time_span: 10,
                client_id: "client1".into(),
                topic_name: "topicA".into(),
            },
            SlowSubscribeKey {
                time_span: 5,
                client_id: "client3".into(),
                topic_name: "topicD".into(),
            },
        ];

        // Use the keys() method from ConcurrentBTreeMap which returns keys in reverse order
        let actual_order = manager.slow_subscribe_info.keys_reverse();

        assert_eq!(actual_order, expected_order);
    }

    #[test]
    fn test_slow_subscribe_info_min_key_operations() {
        let manager = MetricsCacheManager::new();

        // Test empty case
        assert_eq!(manager.slow_subscribe_info.min_key(), None);
        assert_eq!(manager.slow_subscribe_info.min_key_value(), None);

        // Insert test data with different time_spans
        // Note: SlowSubscribeKey is ordered by time_span (descending), then client_id, then topic_name
        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 50,
                client_id: "client_b".into(),
                topic_name: "topic_x".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_50".to_string(),
                client_id: "client_b".to_string(),
                topic_name: "topic_x".to_string(),
                node_info: "node_1".to_string(),
                time_span: 1000,
                create_time: 1000,
            },
        );

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 100,
                client_id: "client_a".into(),
                topic_name: "topic_z".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_100".to_string(),
                client_id: "client_a".to_string(),
                topic_name: "topic_z".to_string(),
                node_info: "node_2".to_string(),
                time_span: 2000,
                create_time: 2000,
            },
        );

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 75,
                client_id: "client_c".into(),
                topic_name: "topic_y".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_75".to_string(),
                client_id: "client_c".to_string(),
                topic_name: "topic_y".to_string(),
                node_info: "node_3".to_string(),
                time_span: 1500,
                create_time: 1500,
            },
        );

        // Test same time_span with different client_id
        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 100,
                client_id: "client_b".into(),
                topic_name: "topic_a".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_100_b".to_string(),
                client_id: "client_b".to_string(),
                topic_name: "topic_a".to_string(),
                node_info: "node_4".to_string(),
                time_span: 2100,
                create_time: 2100,
            },
        );

        // The "minimum" key in BTreeMap ordering is the one with highest time_span (100),
        // and among those with same time_span, the one with smallest client_id ("client_a")
        let expected_min_key = SlowSubscribeKey {
            time_span: 50,
            client_id: "client_b".to_string(),
            topic_name: "topic_x".to_string(),
        };

        let expected_min_data = SlowSubscribeData {
            subscribe_name: "subscribe_50".to_string(),
            client_id: "client_b".to_string(),
            topic_name: "topic_x".to_string(),
            node_info: "node_1".to_string(),
            time_span: 1000,
            create_time: 1000,
        };

        // Test min_key - should return the most concerning slow subscribe (highest time_span)
        assert_eq!(
            manager.slow_subscribe_info.min_key(),
            Some(expected_min_key.clone())
        );

        // Test min_key_value
        assert_eq!(
            manager.slow_subscribe_info.min_key_value(),
            Some((expected_min_key.clone(), expected_min_data))
        );

        // Remove the "minimum" key (highest time_span) and test again
        let removed_data = manager.slow_subscribe_info.remove(&expected_min_key);
        assert!(removed_data.is_some());

        // Now the minimum should be the other key with time_span 100
        let new_expected_min_key = SlowSubscribeKey {
            time_span: 75,
            client_id: "client_c".into(),
            topic_name: "topic_y".into(),
        };

        assert_eq!(
            manager.slow_subscribe_info.min_key(),
            Some(new_expected_min_key.clone())
        );

        // Clear all and test empty case again
        manager.slow_subscribe_info.clear();
        assert_eq!(manager.slow_subscribe_info.min_key(), None);
        assert_eq!(manager.slow_subscribe_info.min_key_value(), None);
    }

    #[test]
    fn test_slow_subscribe_info_max_key_operations() {
        let manager = MetricsCacheManager::new();

        // Test empty case
        assert_eq!(manager.slow_subscribe_info.max_key(), None);
        assert_eq!(manager.slow_subscribe_info.max_key_value(), None);

        // Insert test data with different time_spans
        // Note: SlowSubscribeKey is ordered by time_span (ascending), then client_id, then topic_name
        // So max_key() returns the key with the HIGHEST time_span (most concerning slow subscribe)
        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 50,
                client_id: "client_b".into(),
                topic_name: "topic_x".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_50".to_string(),
                client_id: "client_b".to_string(),
                topic_name: "topic_x".to_string(),
                node_info: "node_1".to_string(),
                time_span: 1000,
                create_time: 1000,
            },
        );

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 100,
                client_id: "client_a".into(),
                topic_name: "topic_z".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_100".to_string(),
                client_id: "client_a".to_string(),
                topic_name: "topic_z".to_string(),
                node_info: "node_2".to_string(),
                time_span: 2000,
                create_time: 2000,
            },
        );

        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 75,
                client_id: "client_c".into(),
                topic_name: "topic_y".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_75".to_string(),
                client_id: "client_c".to_string(),
                topic_name: "topic_y".to_string(),
                node_info: "node_3".to_string(),
                time_span: 1500,
                create_time: 1500,
            },
        );

        // Test same time_span with different client_id
        manager.record_slow_subscribe_info(
            SlowSubscribeKey {
                time_span: 100,
                client_id: "client_b".into(),
                topic_name: "topic_a".into(),
            },
            SlowSubscribeData {
                subscribe_name: "subscribe_100_b".to_string(),
                client_id: "client_b".to_string(),
                topic_name: "topic_a".to_string(),
                node_info: "node_4".to_string(),
                time_span: 2100,
                create_time: 2100,
            },
        );

        // The "maximum" key in BTreeMap ordering is the one with highest time_span (100),
        // and among those with same time_span, the one with largest client_id and topic_name
        let expected_max_key = SlowSubscribeKey {
            time_span: 100,
            client_id: "client_b".to_string(),
            topic_name: "topic_a".to_string(),
        };

        let expected_max_data = SlowSubscribeData {
            subscribe_name: "subscribe_100_b".to_string(),
            client_id: "client_b".to_string(),
            topic_name: "topic_a".to_string(),
            node_info: "node_4".to_string(),
            time_span: 2100,
            create_time: 2100,
        };

        // Test max_key - should return the most concerning slow subscribe (highest time_span)
        assert_eq!(
            manager.slow_subscribe_info.max_key(),
            Some(expected_max_key.clone())
        );

        // Test max_key_value
        assert_eq!(
            manager.slow_subscribe_info.max_key_value(),
            Some((expected_max_key.clone(), expected_max_data))
        );

        // Remove the "maximum" key and test again
        let removed_data = manager.slow_subscribe_info.remove(&expected_max_key);
        assert!(removed_data.is_some());

        // Now the maximum should be the other key with time_span 100
        let new_expected_max_key = SlowSubscribeKey {
            time_span: 100,
            client_id: "client_a".into(),
            topic_name: "topic_z".into(),
        };

        assert_eq!(
            manager.slow_subscribe_info.max_key(),
            Some(new_expected_max_key.clone())
        );

        // Clear all and test empty case again
        manager.slow_subscribe_info.clear();
        assert_eq!(manager.slow_subscribe_info.max_key(), None);
        assert_eq!(manager.slow_subscribe_info.max_key_value(), None);
    }

    #[test]
    fn test_slow_subscribe_index_remove() {
        let manager = MetricsCacheManager::new();

        // Test remove from empty index
        let result = manager.remove_slow_subscribe_index("client1".into(), "topicA".into());
        assert_eq!(result, None);

        // Insert a value
        manager.record_slow_subscribe_index("client1".into(), "topicA".into(), 10);

        // Verify it exists
        let result = manager.get_slow_subscribe_index_value("client1".into(), "topicA".into());
        assert_eq!(result, Some((10, "client1".into(), "topicA".into())));

        // Remove the value
        let removed = manager.remove_slow_subscribe_index("client1".into(), "topicA".into());
        assert_eq!(removed, Some((10, "client1".into(), "topicA".into())));

        // Verify it's gone
        let result = manager.get_slow_subscribe_index_value("client1".into(), "topicA".into());
        assert_eq!(result, None);

        // Try to remove again
        let result = manager.remove_slow_subscribe_index("client1".into(), "topicA".into());
        assert_eq!(result, None);
    }

    #[test]
    fn test_slow_subscribe_index_remove_multiple() {
        let manager = MetricsCacheManager::new();

        // Insert multiple values
        manager.record_slow_subscribe_index("client1".into(), "topicA".into(), 10);
        manager.record_slow_subscribe_index("client1".into(), "topicB".into(), 20);
        manager.record_slow_subscribe_index("client2".into(), "topicA".into(), 30);

        // Remove one specific entry
        let removed = manager.remove_slow_subscribe_index("client1".into(), "topicA".into());
        assert_eq!(removed, Some((10, "client1".into(), "topicA".into())));

        // Verify others still exist
        let result1 = manager.get_slow_subscribe_index_value("client1".into(), "topicB".into());
        assert_eq!(result1, Some((20, "client1".into(), "topicB".into())));

        let result2 = manager.get_slow_subscribe_index_value("client2".into(), "topicA".into());
        assert_eq!(result2, Some((30, "client2".into(), "topicA".into())));

        // Verify removed entry is gone
        let result3 = manager.get_slow_subscribe_index_value("client1".into(), "topicA".into());
        assert_eq!(result3, None);
    }

    #[test]
    fn test_slow_subscribe_info_remove() {
        let manager = MetricsCacheManager::new();

        let key = SlowSubscribeKey {
            time_span: 10,
            client_id: "client1".into(),
            topic_name: "topicA".into(),
        };

        // Test remove from empty info
        let result = manager.remove_slow_subscribe_info(&key);
        assert_eq!(result, None);

        // Insert a value
        let data = SlowSubscribeData {
            subscribe_name: "subscribe_name".to_string(),
            client_id: "client1".to_string(),
            topic_name: "topicA".to_string(),
            node_info: "node_info".to_string(),
            time_span: 100,
            create_time: 123456789,
        };
        manager.record_slow_subscribe_info(key.clone(), data.clone());

        // Verify it exists
        let result = manager.get_slow_subscribe_info(key.clone());
        assert_eq!(result, Some(data.clone()));

        // Remove the value
        let removed = manager.remove_slow_subscribe_info(&key);
        assert_eq!(removed, Some(data));

        // Verify it's gone
        let result = manager.get_slow_subscribe_info(key.clone());
        assert_eq!(result, None);

        // Try to remove again
        let result = manager.remove_slow_subscribe_info(&key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_slow_subscribe_info_remove_multiple() {
        let manager = MetricsCacheManager::new();

        let key1 = SlowSubscribeKey {
            time_span: 10,
            client_id: "client1".into(),
            topic_name: "topicA".into(),
        };

        let key2 = SlowSubscribeKey {
            time_span: 20,
            client_id: "client2".into(),
            topic_name: "topicB".into(),
        };

        let key3 = SlowSubscribeKey {
            time_span: 30,
            client_id: "client3".into(),
            topic_name: "topicC".into(),
        };

        let data1 = SlowSubscribeData {
            subscribe_name: "subscribe1".to_string(),
            client_id: "client1".to_string(),
            topic_name: "topicA".to_string(),
            node_info: "node1".to_string(),
            time_span: 100,
            create_time: 111,
        };

        let data2 = SlowSubscribeData {
            subscribe_name: "subscribe2".to_string(),
            client_id: "client2".to_string(),
            topic_name: "topicB".to_string(),
            node_info: "node2".to_string(),
            time_span: 200,
            create_time: 222,
        };

        let data3 = SlowSubscribeData {
            subscribe_name: "subscribe3".to_string(),
            client_id: "client3".to_string(),
            topic_name: "topicC".to_string(),
            node_info: "node3".to_string(),
            time_span: 300,
            create_time: 333,
        };

        // Insert multiple values
        manager.record_slow_subscribe_info(key1.clone(), data1.clone());
        manager.record_slow_subscribe_info(key2.clone(), data2.clone());
        manager.record_slow_subscribe_info(key3.clone(), data3.clone());

        // Remove one specific entry
        let removed = manager.remove_slow_subscribe_info(&key2);
        assert_eq!(removed, Some(data2));

        // Verify others still exist
        let result1 = manager.get_slow_subscribe_info(key1.clone());
        assert_eq!(result1, Some(data1));

        let result3 = manager.get_slow_subscribe_info(key3.clone());
        assert_eq!(result3, Some(data3));

        // Verify removed entry is gone
        let result2 = manager.get_slow_subscribe_info(key2);
        assert_eq!(result2, None);
    }
}
