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
use common_base::tools::{loop_select_ticket, now_second};
use common_metrics::mqtt::publish::{
    record_messages_dropped_no_subscribers_get, record_mqtt_messages_received_get,
    record_mqtt_messages_sent_get,
};
use common_metrics::mqtt::statistics::{
    record_mqtt_connections_set, record_mqtt_sessions_set, record_mqtt_subscribers_set,
    record_mqtt_subscriptions_shared_set, record_mqtt_topics_set,
};
use common_metrics::mqtt::topic::{get_topic_messages_sent, get_topic_messages_written};
use dashmap::DashMap;
use network_server::common::connection_manager::ConnectionManager;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tracing::info;

#[derive(Default, Clone)]
pub struct MetricsCacheManager {
    pub connection_num: DashMap<u64, u64>,
    pub topic_num: DashMap<u64, u64>,
    pub subscribe_num: DashMap<u64, u64>,
    // message in
    pub message_in_num: DashMap<u64, u64>,
    pub pre_message_in_num: Arc<RwLock<u64>>,

    // message out
    pub message_out_num: DashMap<u64, u64>,
    pub pre_message_out_num: Arc<RwLock<u64>>,

    // message drop
    pub message_drop_num: DashMap<u64, u64>,
    pub pre_message_drop_num: Arc<RwLock<u64>>,

    // topic in
    pub topic_in_num: DashMap<String, DashMap<u64, u64>>,
    pub pre_topic_in_num: DashMap<String, u64>,

    // topic out
    pub topic_out_num: DashMap<String, DashMap<u64, u64>>,
    pub pre_topic_out_num: DashMap<String, u64>,

    // subscribe out
    pub subscribe_out_num: DashMap<String, DashMap<u64, u64>>,
    pub pre_subscribe_out_num: DashMap<String, u64>,
}

impl MetricsCacheManager {
    pub fn new() -> Self {
        MetricsCacheManager {
            connection_num: DashMap::with_capacity(4),
            topic_num: DashMap::with_capacity(4),
            subscribe_num: DashMap::with_capacity(4),
            message_in_num: DashMap::with_capacity(4),
            pre_message_in_num: Arc::new(RwLock::new(0)),
            message_out_num: DashMap::with_capacity(4),
            pre_message_out_num: Arc::new(RwLock::new(0)),
            message_drop_num: DashMap::with_capacity(4),
            pre_message_drop_num: Arc::new(RwLock::new(0)),
            topic_in_num: DashMap::with_capacity(4),
            pre_topic_in_num: DashMap::with_capacity(2),
            topic_out_num: DashMap::with_capacity(4),
            pre_topic_out_num: DashMap::with_capacity(2),
            subscribe_out_num: DashMap::with_capacity(4),
            pre_subscribe_out_num: DashMap::with_capacity(2),
        }
    }

    // connection num / topic num / subscribe num
    pub fn record_connection_num(&self, time: u64, num: u64) {
        self.connection_num.insert(time, num);
    }

    pub fn record_topic_num(&self, time: u64, num: u64) {
        self.topic_num.insert(time, num);
    }

    pub fn record_subscribe_num(&self, time: u64, num: u64) {
        self.subscribe_num.insert(time, num);
    }

    // message in
    pub async fn record_message_in_num(&self, time: u64, total: u64, num: u64) {
        self.message_in_num.insert(time, num);
        let mut val = self.pre_message_in_num.write().await;
        *val = total;
    }
    pub async fn get_pre_message_in(&self) -> u64 {
        let data = self.pre_message_in_num.read().await;
        *data
    }

    // message out
    pub async fn record_message_out_num(&self, time: u64, total: u64, num: u64) {
        self.message_out_num.insert(time, num);
        let mut val = self.pre_message_out_num.write().await;
        *val = total;
    }

    pub async fn get_pre_message_out(&self) -> u64 {
        let data = self.pre_message_out_num.read().await;
        *data
    }

    // message drop
    pub async fn record_message_drop_num(&self, time: u64, total: u64, num: u64) {
        self.message_drop_num.insert(time, num);
        let mut val = self.pre_message_drop_num.write().await;
        *val = total;
    }

    pub async fn get_pre_message_drop(&self) -> u64 {
        let data = self.pre_message_drop_num.read().await;
        *data
    }

    // topic in
    pub fn record_topic_in_num(&self, topic: &str, time: u64, total: u64, num: u64) {
        // num
        if let Some(data) = self.topic_in_num.get_mut(topic) {
            data.insert(time, num);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(time, num);
            self.topic_in_num.insert(topic.to_string(), data);
        }

        // total num
        self.pre_topic_in_num.insert(topic.to_string(), total);
    }

    pub fn get_topic_in_num(&self, topic: &str) -> DashMap<u64, u64> {
        if let Some(data) = self.topic_in_num.get(topic) {
            data.clone()
        } else {
            DashMap::new()
        }
    }

    pub async fn get_topic_in_pre_total(&self, topic: &str, num: u64) -> u64 {
        if let Some(data) = self.pre_topic_in_num.get(topic) {
            *data
        } else {
            num
        }
    }

    // topic out
    pub fn record_topic_out_num(&self, topic: &str, time: u64, total: u64, num: u64) {
        // num
        if let Some(data) = self.topic_out_num.get_mut(topic) {
            data.insert(time, num);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(time, num);
            self.topic_out_num.insert(topic.to_string(), data);
        }

        // total num
        self.pre_topic_out_num.insert(topic.to_string(), total);
    }

    pub fn get_topic_out_num(&self, topic: &str) -> DashMap<u64, u64> {
        if let Some(data) = self.topic_out_num.get(topic) {
            data.clone()
        } else {
            DashMap::new()
        }
    }

    pub async fn get_topic_out_pre_total(&self, topic: &str, num: u64) -> u64 {
        if let Some(data) = self.pre_topic_out_num.get(topic) {
            *data
        } else {
            num
        }
    }

    pub fn convert_monitor_data(&self, data_list: DashMap<u64, u64>) -> Vec<HashMap<String, u64>> {
        let mut results = Vec::new();
        for (time, value) in data_list {
            let mut raw = HashMap::new();
            raw.insert("date".to_string(), time);
            raw.insert("value".to_string(), value);
            results.push(raw);
        }
        results
    }

    // gc
    pub fn gc(&self) {
        let now_time = now_second();
        let save_time = 3600;

        // connection_num/topic_num/subscribe_num
        for data in self.connection_num.iter() {
            if (data.key() + save_time) < now_time {
                self.connection_num.remove(data.key());
            }
        }

        for data in self.topic_num.iter() {
            if (data.key() + save_time) < now_time {
                self.topic_num.remove(data.key());
            }
        }

        for data in self.subscribe_num.iter() {
            if (data.key() + save_time) < now_time {
                self.subscribe_num.remove(data.key());
            }
        }

        // message_in_num
        for data in self.message_in_num.iter() {
            if (data.key() + save_time) < now_time {
                self.message_in_num.remove(data.key());
            }
        }

        // message_out_num
        for data in self.message_out_num.iter() {
            if (data.key() + save_time) < now_time {
                self.message_out_num.remove(data.key());
            }
        }

        // message_drop_num
        for data in self.message_drop_num.iter() {
            if (data.key() + save_time) < now_time {
                self.message_drop_num.remove(data.key());
            }
        }

        // topic in
        for raw in self.topic_in_num.iter() {
            for data in raw.iter_mut() {
                if (data.key() + save_time) < now_time {
                    raw.remove(data.key());
                }
            }
        }

        // topic out
        for raw in self.topic_out_num.iter() {
            for data in raw.iter_mut() {
                if (data.key() + save_time) < now_time {
                    raw.remove(data.key());
                }
            }
        }
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
    let raw_metrics_cache_manager = metrics_cache_manager.clone();
    let raw_cache_manager = cache_manager.clone();
    let raw_stop_send = stop_send.clone();
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
            let now: u64 = now_second();

            // connection num / topic num / subscribe num
            raw_metrics_cache_manager
                .record_connection_num(now, connection_manager.connections.len() as u64);
            raw_metrics_cache_manager
                .record_topic_num(now, raw_cache_manager.topic_info.len() as u64);
            raw_metrics_cache_manager
                .record_subscribe_num(now, subscribe_manager.list_subscribe().len() as u64);

            // message in
            let message_in = record_mqtt_messages_received_get();
            let pre_message_in = raw_metrics_cache_manager.get_pre_message_in().await;
            raw_metrics_cache_manager
                .record_message_in_num(
                    now,
                    message_in,
                    calc_value(message_in, pre_message_in, time_window),
                )
                .await;

            // message out
            let message_out = record_mqtt_messages_sent_get();
            let pre_message_out = raw_metrics_cache_manager.get_pre_message_out().await;
            raw_metrics_cache_manager
                .record_message_out_num(
                    now,
                    message_out,
                    calc_value(message_out, pre_message_out, time_window),
                )
                .await;

            // message drop
            let message_drop = record_messages_dropped_no_subscribers_get();
            let pre_message_drop = raw_metrics_cache_manager.get_pre_message_drop().await;
            raw_metrics_cache_manager
                .record_message_drop_num(
                    now,
                    message_drop,
                    calc_value(message_drop, pre_message_drop, time_window),
                )
                .await;

            // Many system metrics can be reused here. We only need to get the instantaneous value.
            // However, it should be noted that prometheus export itself is periodic,
            // and the current function is also periodic.
            // Further, we can conclude that the time range of
            // indicator export is [min(metrics_export_interval,time_window), metrics_export_interval + time_window]
            record_mqtt_connections_set(connection_manager.connections.len() as i64);
            record_mqtt_sessions_set(raw_cache_manager.session_info.len() as i64);
            record_mqtt_topics_set(raw_cache_manager.topic_info.len() as i64);
            record_mqtt_subscribers_set(subscribe_manager.list_subscribe().len() as i64);
            record_mqtt_subscriptions_shared_set(
                subscribe_manager.share_leader_push_list().len() as i64
            );
            Ok(())
        };
        loop_select_ticket(record_func, time_window, &raw_stop_send).await;
    });

    let raw_cache_manager = cache_manager.clone();
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
            let now: u64 = now_second();

            for topic in raw_cache_manager.get_all_topic_name() {
                // topic in
                let message_in = get_topic_messages_written(&topic);
                let pre_message_in = metrics_cache_manager
                    .get_topic_in_pre_total(&topic, message_in)
                    .await;
                metrics_cache_manager
                    .record_message_out_num(
                        now,
                        message_in,
                        calc_value(pre_message_in, message_in, time_window),
                    )
                    .await;

                // topic out
                let message_in = get_topic_messages_sent(&topic);
                let pre_message_in = metrics_cache_manager
                    .get_topic_in_pre_total(&topic, message_in)
                    .await;
                metrics_cache_manager
                    .record_message_out_num(
                        now,
                        message_in,
                        calc_value(pre_message_in, message_in, time_window),
                    )
                    .await;
            }

            Ok(())
        };
        loop_select_ticket(record_func, time_window, &stop_send).await;
    });
}

pub fn metrics_gc_thread(
    metrics_cache_manager: Arc<MetricsCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    info!("Metrics gc thread start successfully");
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
            metrics_cache_manager.gc();
            Ok(())
        };
        loop_select_ticket(record_func, 3600, &stop_send).await;
    });
}

fn calc_value(max_value: u64, min_value: u64, time_window: u64) -> u64 {
    (max_value - min_value) / time_window
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
        let cache_manager = test_build_mqtt_cache_manager().await;
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

        assert_eq!(
            metrics_cache_manager
                .convert_monitor_data(metrics_cache_manager.connection_num.clone())
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .convert_monitor_data(metrics_cache_manager.topic_num.clone())
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .convert_monitor_data(metrics_cache_manager.subscribe_num.clone())
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .convert_monitor_data(metrics_cache_manager.message_in_num.clone())
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .convert_monitor_data(metrics_cache_manager.message_out_num.clone())
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .convert_monitor_data(metrics_cache_manager.message_drop_num.clone())
                .len(),
            7
        );
    }
}
