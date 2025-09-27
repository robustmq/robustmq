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

use crate::{handler::cache::MQTTCacheManager, subscribe::manager::SubscribeManager};
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select, now_second};
use common_metrics::mqtt::statistics::{
    record_mqtt_connections_set, record_mqtt_sessions_set, record_mqtt_subscribers_set,
    record_mqtt_subscriptions_shared_set, record_mqtt_topics_set,
};
use common_metrics::network::{record_broker_connections_max, record_broker_connections_num};
use dashmap::DashMap;
use network_server::common::connection_manager::ConnectionManager;
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
        }
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
    cache_manager: Arc<MQTTCacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    time_window: u64,
    stop_send: broadcast::Sender<bool>,
) {
    info!("Metrics record thread start successfully");
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
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

            // record metrics
            record_mqtt_connections_set(connection_manager.connections.len() as i64);
            record_mqtt_sessions_set(cache_manager.session_info.len() as i64);
            record_mqtt_topics_set(cache_manager.topic_info.len() as i64);
            record_mqtt_subscribers_set(subscribe_manager.subscribe_list.len() as i64);
            record_mqtt_subscriptions_shared_set(subscribe_manager.share_leader_push.len() as i64);
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
        let record_func = async || -> ResultCommonError {
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
    use crate::common::tool::test_build_mqtt_cache_manager;
    use std::{
        net::{Ipv4Addr, SocketAddrV4},
        sync::Arc,
        time::Duration,
    };

    use crate::{
        common::metrics_cache::{metrics_gc_thread, metrics_record_thread, MetricsCacheManager},
        subscribe::manager::SubscribeManager,
    };
    use common_base::tools::now_second;
    use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
    use network_server::common::connection_manager::ConnectionManager;
    use tokio::{sync::broadcast, time::sleep};

    #[tokio::test]
    pub async fn minute_test() {
        let mut times = 0;
        loop {
            if times >= 1 {
                break;
            }
            let now = now_second();
            if now.is_multiple_of(60) {
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
        let cache_manager = test_build_mqtt_cache_manager();
        let subscribe_manager = Arc::new(SubscribeManager::new());

        // add mock connection
        let connection_mgr = ConnectionManager::new(3, 1000);
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
}
