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
    pub data_num: DashMap<String, DashMap<u64, u64>>,
    pub pre_data_num: DashMap<String, u64>,
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

        for raw in self.data_num.iter() {
            for data in raw.value().iter() {
                if (data.key() + save_time) < now_time {
                    raw.value().remove(data.key());
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

    // common
    let raw_metrics_cache_manager = metrics_cache_manager.clone();
    let raw_cache_manager = cache_manager.clone();
    let raw_subscribe_manager = subscribe_manager.clone();
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
                .record_subscribe_num(now, raw_subscribe_manager.list_subscribe().len() as u64);

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
            record_mqtt_subscribers_set(raw_subscribe_manager.list_subscribe().len() as i64);
            record_mqtt_subscriptions_shared_set(
                raw_subscribe_manager.share_leader_push_list().len() as i64,
            );
            Ok(())
        };
        loop_select_ticket(record_func, time_window, &raw_stop_send).await;
    });

    // topic
    let raw_cache_manager = cache_manager.clone();
    let raw_metrics_cache_manager = metrics_cache_manager.clone();
    let raw_stop_send = stop_send.clone();
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
            let now: u64 = now_second();

            for topic in raw_cache_manager.get_all_topic_name() {
                // topic in
                let message_in = get_topic_messages_written(&topic);
                let pre_message_in =
                    raw_metrics_cache_manager.get_topic_in_pre_total(&topic, message_in);
                raw_metrics_cache_manager
                    .record_message_out_num(
                        now,
                        message_in,
                        calc_value(pre_message_in, message_in, time_window),
                    )
                    .await;

                // topic out
                let message_in = get_topic_messages_sent(&topic);
                let pre_message_in =
                    raw_metrics_cache_manager.get_topic_in_pre_total(&topic, message_in);
                raw_metrics_cache_manager
                    .record_message_out_num(
                        now,
                        message_in,
                        calc_value(pre_message_in, message_in, time_window),
                    )
                    .await;
            }

            Ok(())
        };
        loop_select_ticket(record_func, time_window, &raw_stop_send).await;
    });

    // subscribe
    let raw_subscribe_manager = subscribe_manager.clone();
    let raw_metrics_cache_manager = metrics_cache_manager.clone();
    let raw_stop_send = stop_send.clone();
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
            let now: u64 = now_second();

            for (_, sub) in raw_subscribe_manager.list_subscribe() {
                // subscribe send success
                let num = get_subscribe_messages_sent(&sub.client_id, &sub.path, true);
                let pre_num = raw_metrics_cache_manager.get_subscribe_send_pre_total(
                    &sub.client_id,
                    &sub.path,
                    true,
                    num,
                );
                raw_metrics_cache_manager.record_subscribe_send_num(
                    &sub.client_id,
                    &sub.path,
                    true,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                );

                // subscribe send failure
                let num = get_subscribe_messages_sent(&sub.client_id, &sub.path, false);
                let pre_num = raw_metrics_cache_manager.get_subscribe_send_pre_total(
                    &sub.client_id,
                    &sub.path,
                    false,
                    num,
                );
                raw_metrics_cache_manager.record_subscribe_send_num(
                    &sub.client_id,
                    &sub.path,
                    false,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                );
            }

            // exclusive push
            for (_, sub) in raw_subscribe_manager.exclusive_push_list() {
                // success
                let num = get_subscribe_topic_messages_sent(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true,
                );

                let pre_num = raw_metrics_cache_manager.get_subscribe_topic_send_pre_total(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true,
                    num,
                );
                raw_metrics_cache_manager.record_subscribe_topic_send_num(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                );

                // failure
                let num = get_subscribe_topic_messages_sent(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false,
                );

                let pre_num = raw_metrics_cache_manager.get_subscribe_topic_send_pre_total(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false,
                    num,
                );
                raw_metrics_cache_manager.record_subscribe_topic_send_num(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                );
            }

            // exclusive push
            for (_, _) in raw_subscribe_manager.share_leader_push_list() {
                // todo
            }

            Ok(())
        };
        loop_select_ticket(record_func, time_window, &raw_stop_send).await;
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
        common::metrics_cache::{
            metrics_gc_thread, metrics_record_thread, MetricsCacheManager,
            METRICS_TYPE_KEY_CONNECTION_NUM, METRICS_TYPE_KEY_SUBSCRIBE_NUM,
            METRICS_TYPE_KEY_TOPIC_NUM,
        },
        subscribe::manager::SubscribeManager,
    };
    use common_base::tools::now_second;
    use dashmap::DashMap;
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

        // Get data from data_num map
        let connection_data = metrics_cache_manager
            .data_num
            .get(METRICS_TYPE_KEY_CONNECTION_NUM)
            .map(|v| v.clone())
            .unwrap_or_else(DashMap::new);
        let topic_data = metrics_cache_manager
            .data_num
            .get(METRICS_TYPE_KEY_TOPIC_NUM)
            .map(|v| v.clone())
            .unwrap_or_else(DashMap::new);
        let subscribe_data = metrics_cache_manager
            .data_num
            .get(METRICS_TYPE_KEY_SUBSCRIBE_NUM)
            .map(|v| v.clone())
            .unwrap_or_else(DashMap::new);

        assert_eq!(connection_data.len(), 10);
        assert_eq!(topic_data.len(), 10);
        assert_eq!(subscribe_data.len(), 10);

        assert_eq!(
            metrics_cache_manager
                .convert_monitor_data(connection_data)
                .len(),
            7
        );
        assert_eq!(
            metrics_cache_manager.convert_monitor_data(topic_data).len(),
            7
        );
        assert_eq!(
            metrics_cache_manager
                .convert_monitor_data(subscribe_data)
                .len(),
            7
        );
    }
}
