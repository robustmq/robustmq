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
use common_metrics::mqtt::subscribe::{
    get_subscribe_messages_sent, get_subscribe_topic_messages_sent,
};
use common_metrics::mqtt::topic::{get_topic_messages_sent, get_topic_messages_written};
use dashmap::DashMap;
use network_server::common::connection_manager::ConnectionManager;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tracing::info;

pub const METRICS_TYPE_KEY_CONNECTION_NUM: &str = "connection_num";
pub const METRICS_TYPE_KEY_TOPIC_NUM: &str = "topic_num";
pub const METRICS_TYPE_KEY_SUBSCRIBE_NUM: &str = "subscribe_num";
pub const METRICS_TYPE_KEY_MESSAGE_IN_NUM: &str = "message_in";
pub const METRICS_TYPE_KEY_MESSAGE_OUT_NUM: &str = "message_out";
pub const METRICS_TYPE_KEY_MESSAGE_DROP_NUM: &str = "message_drop";
pub const METRICS_TYPE_KEY_TOPIC_IN_NUM: &str = "topic_in";
pub const METRICS_TYPE_KEY_TOPIC_OUT_NUM: &str = "topic_out";
pub const METRICS_TYPE_KEY_SUBSCRIBE_SEND: &str = "subscribe_send";
pub const METRICS_TYPE_KEY_SUBSCRIBE_TOPIC_SEND: &str = "subscribe_topic_send";

#[derive(Default, Clone)]
pub struct MetricsCacheManager {
    data_num: DashMap<String, DashMap<u64, u64>>,
    pre_data_num: DashMap<String, u64>,
}

impl MetricsCacheManager {
    pub fn new() -> Self {
        MetricsCacheManager {
            data_num: DashMap::with_capacity(4),
            pre_data_num: DashMap::with_capacity(4),
        }
    }

    // connection num / topic num / subscribe num
    pub fn record_connection_num(&self, time: u64, num: u64) {
        self.record_num(METRICS_TYPE_KEY_CONNECTION_NUM, time, num);
    }

    pub fn get_connection_num(&self) -> DashMap<u64, u64> {
        self.data_num
            .get(METRICS_TYPE_KEY_CONNECTION_NUM)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn record_topic_num(&self, time: u64, num: u64) {
        self.record_num(METRICS_TYPE_KEY_TOPIC_NUM, time, num);
    }

    pub fn get_topic_num(&self) -> DashMap<u64, u64> {
        self.data_num
            .get(METRICS_TYPE_KEY_TOPIC_NUM)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn record_subscribe_num(&self, time: u64, num: u64) {
        self.record_num(METRICS_TYPE_KEY_SUBSCRIBE_NUM, time, num);
    }

    pub fn get_subscribe_num(&self) -> DashMap<u64, u64> {
        self.data_num
            .get(METRICS_TYPE_KEY_SUBSCRIBE_NUM)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    fn record_num(&self, key: &str, time: u64, num: u64) {
        if let Some(data) = self.data_num.get_mut(key) {
            data.insert(time, num);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(time, num);
            self.data_num.insert(key.to_string(), data);
        }
    }

    fn record_pre_num(&self, key: &str, total: u64) {
        self.pre_data_num.insert(key.to_string(), total);
    }

    fn get_pre_num(&self, key: &str) -> u64 {
        self.pre_data_num.get(key).map(|v| *v).unwrap_or(0)
    }

    // message in
    pub async fn record_message_in_num(&self, time: u64, total: u64, num: u64) {
        self.record_num(METRICS_TYPE_KEY_MESSAGE_IN_NUM, time, num);
        self.record_pre_num(METRICS_TYPE_KEY_MESSAGE_IN_NUM, total);
    }

    pub fn get_message_in_num(&self) -> DashMap<u64, u64> {
        self.data_num
            .get(METRICS_TYPE_KEY_MESSAGE_IN_NUM)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub async fn get_pre_message_in(&self) -> u64 {
        self.get_pre_num(METRICS_TYPE_KEY_MESSAGE_IN_NUM)
    }

    pub fn get_message_in_rate(&self) -> u64 {
        let data = self.get_message_in_num();
        get_max_key_value(&data)
    }

    // message out
    pub async fn record_message_out_num(&self, time: u64, total: u64, num: u64) {
        self.record_num(METRICS_TYPE_KEY_MESSAGE_OUT_NUM, time, num);
        self.record_pre_num(METRICS_TYPE_KEY_MESSAGE_OUT_NUM, total);
    }

    pub fn get_message_out_num(&self) -> DashMap<u64, u64> {
        self.data_num
            .get(METRICS_TYPE_KEY_MESSAGE_OUT_NUM)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub async fn get_pre_message_out(&self) -> u64 {
        self.get_pre_num(METRICS_TYPE_KEY_MESSAGE_OUT_NUM)
    }

    pub fn get_message_out_rate(&self) -> u64 {
        let data = self.get_message_out_num();
        get_max_key_value(&data)
    }

    // message drop
    pub async fn record_message_drop_num(&self, time: u64, total: u64, num: u64) {
        self.record_num(METRICS_TYPE_KEY_MESSAGE_DROP_NUM, time, num);
        self.record_pre_num(METRICS_TYPE_KEY_MESSAGE_DROP_NUM, total);
    }

    pub fn get_message_drop_num(&self) -> DashMap<u64, u64> {
        self.data_num
            .get(METRICS_TYPE_KEY_MESSAGE_DROP_NUM)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub async fn get_pre_message_drop(&self) -> u64 {
        self.get_pre_num(METRICS_TYPE_KEY_MESSAGE_DROP_NUM)
    }

    pub fn get_message_drop_rate(&self) -> u64 {
        let data = self.get_message_drop_num();
        get_max_key_value(&data)
    }

    // topic in
    pub fn record_topic_in_num(&self, topic: &str, time: u64, total: u64, num: u64) {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_IN_NUM, topic);
        self.record_num(&key, time, num);
        self.record_pre_num(&key, total);
    }

    pub fn get_topic_in_num(&self, topic: &str) -> DashMap<u64, u64> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_IN_NUM, topic);
        self.data_num
            .get(&key)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn get_topic_in_pre_total(&self, topic: &str, num: u64) -> u64 {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_IN_NUM, topic);
        self.pre_data_num.get(&key).map(|v| *v).unwrap_or(num)
    }

    // topic out
    pub fn record_topic_out_num(&self, topic: &str, time: u64, total: u64, num: u64) {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_OUT_NUM, topic);
        self.record_num(&key, time, num);
        self.record_pre_num(&key, total);
    }

    pub fn get_topic_out_num(&self, topic: &str) -> DashMap<u64, u64> {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_OUT_NUM, topic);
        self.data_num
            .get(&key)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn get_topic_out_pre_total(&self, topic: &str, num: u64) -> u64 {
        let key = format!("{}_{}", METRICS_TYPE_KEY_TOPIC_OUT_NUM, topic);
        self.pre_data_num.get(&key).map(|v| *v).unwrap_or(num)
    }

    // subscribe send
    pub fn record_subscribe_send_num(
        &self,
        client_id: &str,
        path: &str,
        success: bool,
        time: u64,
        total: u64,
        num: u64,
    ) {
        let key = format!(
            "{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_SEND, client_id, path, success
        );
        self.record_num(&key, time, num);
        self.record_pre_num(&key, total);
    }

    pub fn get_subscribe_send_num(
        &self,
        client_id: &str,
        path: &str,
        success: bool,
    ) -> DashMap<u64, u64> {
        let key = format!(
            "{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_SEND, client_id, path, success
        );
        self.data_num
            .get(&key)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn get_subscribe_send_pre_total(
        &self,
        client_id: &str,
        path: &str,
        success: bool,
        num: u64,
    ) -> u64 {
        let key = format!(
            "{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_SEND, client_id, path, success
        );
        self.pre_data_num.get(&key).map(|v| *v).unwrap_or(num)
    }

    // subscribe topic send
    #[allow(clippy::too_many_arguments)]
    pub fn record_subscribe_topic_send_num(
        &self,
        client_id: &str,
        path: &str,
        topic: &str,
        success: bool,
        time: u64,
        total: u64,
        num: u64,
    ) {
        let key = format!(
            "{}_{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_TOPIC_SEND, client_id, path, topic, success
        );
        self.record_num(&key, time, num);
        self.record_pre_num(&key, total);
    }

    pub fn get_subscribe_topic_send_num(
        &self,
        client_id: &str,
        path: &str,
        topic: &str,
        success: bool,
    ) -> DashMap<u64, u64> {
        let key = format!(
            "{}_{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_TOPIC_SEND, client_id, path, topic, success
        );
        self.data_num
            .get(&key)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn get_subscribe_topic_send_pre_total(
        &self,
        client_id: &str,
        path: &str,
        topic: &str,
        success: bool,
        num: u64,
    ) -> u64 {
        let key = format!(
            "{}_{}_{}_{}_{}",
            METRICS_TYPE_KEY_SUBSCRIBE_TOPIC_SEND, client_id, path, topic, success
        );
        self.pre_data_num.get(&key).map(|v| *v).unwrap_or(num)
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

        for (key, value) in self.data_num.clone() {
            for (time, _) in value {
                if (time + save_time) < now_time {
                    if let Some(data) = self.data_num.get_mut(&key) {
                        data.remove(&time);
                    }
                }
            }
        }
    }
}

fn record_basic_metrics_thread(
    metrics_cache_manager: Arc<MetricsCacheManager>,
    cache_manager: Arc<MQTTCacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    time_window: u64,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
            let now: u64 = now_second();

            // connection num
            metrics_cache_manager
                .record_connection_num(now, connection_manager.connections.len() as u64);

            // topic num
            metrics_cache_manager.record_topic_num(now, cache_manager.topic_info.len() as u64);

            // subscribe num
            metrics_cache_manager
                .record_subscribe_num(now, subscribe_manager.list_subscribe().len() as u64);

            // record metrics
            record_mqtt_connections_set(connection_manager.connections.len() as i64);
            record_mqtt_sessions_set(cache_manager.session_info.len() as i64);
            record_mqtt_topics_set(cache_manager.topic_info.len() as i64);
            record_mqtt_subscribers_set(subscribe_manager.list_subscribe().len() as i64);
            record_mqtt_subscriptions_shared_set(
                subscribe_manager.share_leader_push_list().len() as i64
            );

            // message in
            let num = record_mqtt_messages_received_get();
            let pre_num = metrics_cache_manager.get_pre_message_in().await;
            metrics_cache_manager
                .record_message_in_num(now, num, calc_value(num, pre_num, time_window))
                .await;

            // message out
            let num = record_mqtt_messages_sent_get();
            let pre_num = metrics_cache_manager.get_pre_message_out().await;
            println!("messages_sent:{}", num);
            println!("messages_sent pre_num:{}", pre_num);
            metrics_cache_manager
                .record_message_out_num(now, num, calc_value(num, pre_num, time_window))
                .await;

            // message drop
            let num = record_messages_dropped_no_subscribers_get();
            let pre_num = metrics_cache_manager.get_pre_message_drop().await;
            metrics_cache_manager
                .record_message_drop_num(now, num, calc_value(num, pre_num, time_window))
                .await;

            Ok(())
        };
        loop_select_ticket(record_func, time_window, &stop_send).await;
    });
}

fn record_topic_metrics_thread(
    metrics_cache_manager: Arc<MetricsCacheManager>,
    cache_manager: Arc<MQTTCacheManager>,
    time_window: u64,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
            let now: u64 = now_second();

            for topic in cache_manager.get_all_topic_name() {
                // topic in
                let num = get_topic_messages_written(&topic);
                let pre_num = metrics_cache_manager.get_topic_in_pre_total(&topic, num);
                metrics_cache_manager
                    .record_message_in_num(now, pre_num, calc_value(num, pre_num, time_window))
                    .await;

                // topic out
                let num = get_topic_messages_sent(&topic);
                let pre_num = metrics_cache_manager.get_topic_out_pre_total(&topic, num);
                metrics_cache_manager
                    .record_message_out_num(now, num, calc_value(num, pre_num, time_window))
                    .await;
            }

            Ok(())
        };
        loop_select_ticket(record_func, time_window, &stop_send).await;
    });
}

fn record_subscribe_metrics_thread(
    metrics_cache_manager: Arc<MetricsCacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    time_window: u64,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
            let now: u64 = now_second();

            for (_, sub) in subscribe_manager.list_subscribe() {
                let num = get_subscribe_messages_sent(&sub.client_id, &sub.path, true);
                let pre_num = metrics_cache_manager.get_subscribe_send_pre_total(
                    &sub.client_id,
                    &sub.path,
                    true,
                    num,
                );
                metrics_cache_manager.record_subscribe_send_num(
                    &sub.client_id,
                    &sub.path,
                    true,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                );

                let num = get_subscribe_messages_sent(&sub.client_id, &sub.path, false);
                let pre_num = metrics_cache_manager.get_subscribe_send_pre_total(
                    &sub.client_id,
                    &sub.path,
                    false,
                    num,
                );
                metrics_cache_manager.record_subscribe_send_num(
                    &sub.client_id,
                    &sub.path,
                    false,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                );
            }

            for (_, sub) in subscribe_manager.exclusive_push_list() {
                let num = get_subscribe_topic_messages_sent(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true,
                );

                let pre_num = metrics_cache_manager.get_subscribe_topic_send_pre_total(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true,
                    num,
                );
                metrics_cache_manager.record_subscribe_topic_send_num(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                );

                let num = get_subscribe_topic_messages_sent(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false,
                );

                let pre_num = metrics_cache_manager.get_subscribe_topic_send_pre_total(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false,
                    num,
                );
                metrics_cache_manager.record_subscribe_topic_send_num(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                );
            }

            for (_, _) in subscribe_manager.share_leader_push_list() {
                // todo
            }

            Ok(())
        };
        loop_select_ticket(record_func, time_window, &stop_send).await;
    });
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

    record_basic_metrics_thread(
        metrics_cache_manager.clone(),
        cache_manager.clone(),
        subscribe_manager.clone(),
        connection_manager,
        time_window,
        stop_send.clone(),
    );

    record_topic_metrics_thread(
        metrics_cache_manager.clone(),
        cache_manager,
        time_window,
        stop_send.clone(),
    );

    record_subscribe_metrics_thread(
        metrics_cache_manager,
        subscribe_manager,
        time_window,
        stop_send,
    );
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
    if time_window == 0 {
        return 0;
    }

    let diff = (max_value - min_value) as f64;
    let window = time_window as f64;
    let result = diff / window;

    // 四舍五入
    result.round() as u64
}

fn get_max_key_value(data: &DashMap<u64, u64>) -> u64 {
    if data.is_empty() {
        return 0;
    }

    data.iter()
        .max_by_key(|entry| *entry.key())
        .map(|entry| *entry.value())
        .unwrap_or(0)
}

#[cfg(test)]
mod test {
    use crate::common::metrics_cache::{MetricsCacheManager, calc_value, get_max_key_value};
    use common_base::tools::now_second;
    use dashmap::DashMap;

    #[test]
    fn test_get_max_key_value() {
        let data: DashMap<u64, u64> = DashMap::new();
        data.insert(100, 10);
        data.insert(200, 20);
        data.insert(150, 15);
        data.insert(300, 30);
        data.insert(250, 25);

        assert_eq!(get_max_key_value(&data), 30);

        let empty_data: DashMap<u64, u64> = DashMap::new();
        assert_eq!(get_max_key_value(&empty_data), 0);

        let single_data: DashMap<u64, u64> = DashMap::new();
        single_data.insert(100, 42);
        assert_eq!(get_max_key_value(&single_data), 42);
    }

    #[test]
    fn test_calc_value_rounding() {
        assert_eq!(calc_value(100, 0, 10), 10);
        assert_eq!(calc_value(105, 0, 10), 11);
        assert_eq!(calc_value(106, 0, 10), 11);
        assert_eq!(calc_value(109, 0, 10), 11);
        assert_eq!(calc_value(104, 0, 10), 10);
        assert_eq!(calc_value(103, 0, 10), 10);
        assert_eq!(calc_value(0, 0, 10), 0);
        assert_eq!(calc_value(5, 0, 10), 1);
        assert_eq!(calc_value(4, 0, 10), 0);
        assert_eq!(calc_value(100, 0, 0), 0);
        assert_eq!(calc_value(125, 100, 10), 3);
        assert_eq!(calc_value(124, 100, 10), 2);
    }

    #[tokio::test]
    pub async fn test_topic_out_metrics() {
        let metrics_cache_manager = MetricsCacheManager::new();
        let topic_name = "test/topic";
        let now = now_second();

        metrics_cache_manager.record_topic_out_num(topic_name, now, 100, 10);
        metrics_cache_manager.record_topic_out_num(topic_name, now + 1, 110, 10);
        metrics_cache_manager.record_topic_out_num(topic_name, now + 2, 125, 15);

        let topic_out_data = metrics_cache_manager.get_topic_out_num(topic_name);
        assert_eq!(topic_out_data.len(), 3);
        assert_eq!(*topic_out_data.get(&now).unwrap(), 10);

        let pre_total = metrics_cache_manager.get_topic_out_pre_total(topic_name, 0);
        assert_eq!(pre_total, 125);

        let non_exist_topic = "non/exist/topic";
        assert_eq!(
            metrics_cache_manager
                .get_topic_out_num(non_exist_topic)
                .len(),
            0
        );
        assert_eq!(
            metrics_cache_manager.get_topic_out_pre_total(non_exist_topic, 999),
            999
        );
    }

    #[tokio::test]
    pub async fn test_metrics_gc() {
        let metrics_cache_manager = MetricsCacheManager::new();
        let topic_name = "test/gc/topic";
        let now = now_second();

        let old_time = now - 3700;
        metrics_cache_manager.record_topic_out_num(topic_name, old_time, 100, 10);
        metrics_cache_manager.record_topic_out_num(topic_name, now, 200, 20);

        metrics_cache_manager.gc();

        let data = metrics_cache_manager.get_topic_out_num(topic_name);
        assert_eq!(data.len(), 1);
        assert!(data.contains_key(&now));
    }

    #[tokio::test]
    pub async fn test_message_rate_methods() {
        let metrics_cache_manager = MetricsCacheManager::new();
        let now = now_second();

        metrics_cache_manager
            .record_message_in_num(now, 100, 10)
            .await;
        metrics_cache_manager
            .record_message_in_num(now + 1, 120, 20)
            .await;
        metrics_cache_manager
            .record_message_in_num(now + 2, 150, 30)
            .await;

        let in_rate = metrics_cache_manager.get_message_in_rate();
        assert_eq!(in_rate, 30);

        metrics_cache_manager
            .record_message_out_num(now, 50, 5)
            .await;
        metrics_cache_manager
            .record_message_out_num(now + 1, 60, 10)
            .await;
        metrics_cache_manager
            .record_message_out_num(now + 2, 75, 15)
            .await;

        let out_rate = metrics_cache_manager.get_message_out_rate();
        assert_eq!(out_rate, 15);

        metrics_cache_manager
            .record_message_drop_num(now, 10, 1)
            .await;
        metrics_cache_manager
            .record_message_drop_num(now + 1, 12, 2)
            .await;
        metrics_cache_manager
            .record_message_drop_num(now + 2, 15, 3)
            .await;

        let drop_rate = metrics_cache_manager.get_message_drop_rate();
        assert_eq!(drop_rate, 3);
    }
}
