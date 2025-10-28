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
use common_metrics::mqtt::session::{get_session_messages_in, get_session_messages_out};
use common_metrics::mqtt::statistics::{
    record_mqtt_connections_set, record_mqtt_sessions_set, record_mqtt_subscribers_set,
    record_mqtt_subscriptions_shared_set, record_mqtt_topics_set,
};
use common_metrics::mqtt::subscribe::{
    get_subscribe_messages_sent, get_subscribe_topic_messages_sent,
};
use common_metrics::mqtt::topic::{get_topic_messages_sent, get_topic_messages_written};
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::metrics_cache::mqtt::MQTTMetricsCache;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

fn record_basic_metrics_thread(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
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
                .record_connection_num(now, connection_manager.connections.len() as u64)?;

            // topic num
            metrics_cache_manager.record_topic_num(now, cache_manager.topic_info.len() as u64)?;

            // subscribe num
            metrics_cache_manager
                .record_subscribe_num(now, subscribe_manager.list_subscribe().len() as u64)?;

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
            let pre_num = metrics_cache_manager.get_pre_message_in().await?;
            metrics_cache_manager
                .record_message_in_num(now, num, calc_value(num, pre_num, time_window))
                .await?;

            // message out
            let num = record_mqtt_messages_sent_get();
            let pre_num = metrics_cache_manager.get_pre_message_out().await?;
            metrics_cache_manager
                .record_message_out_num(now, num, calc_value(num, pre_num, time_window))
                .await?;

            // message drop
            let num = record_messages_dropped_no_subscribers_get();
            let pre_num = metrics_cache_manager.get_pre_message_drop().await?;
            metrics_cache_manager
                .record_message_drop_num(now, num, calc_value(num, pre_num, time_window))
                .await?;

            Ok(())
        };
        loop_select_ticket(record_func, time_window, &stop_send).await;
    });
}

fn record_topic_metrics_thread(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
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
                let pre_num = metrics_cache_manager
                    .get_topic_in_pre_total(&topic, num)
                    .await?;
                metrics_cache_manager.record_topic_in_num(
                    &topic,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                )?;

                // topic out
                let num = get_topic_messages_sent(&topic);
                let pre_num = metrics_cache_manager
                    .get_topic_out_pre_total(&topic, num)
                    .await?;
                metrics_cache_manager.record_topic_out_num(
                    &topic,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                )?;
            }

            Ok(())
        };
        loop_select_ticket(record_func, time_window, &stop_send).await;
    });
}

fn record_session_metrics_thread(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    cache_manager: Arc<MQTTCacheManager>,
    time_window: u64,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
            let now: u64 = now_second();

            for client_id in cache_manager.get_session_client_id_list() {
                // session in
                let num = get_session_messages_in(&client_id);
                let pre_num = metrics_cache_manager
                    .get_session_in_pre_total(&client_id, num)
                    .await?;
                metrics_cache_manager.record_session_in_num(
                    &client_id,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                )?;

                // session out
                let num = get_session_messages_out(&client_id);
                let pre_num = metrics_cache_manager
                    .get_session_out_pre_total(&client_id, num)
                    .await?;
                metrics_cache_manager.record_session_out_num(
                    &client_id,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                )?;
            }

            Ok(())
        };
        loop_select_ticket(record_func, time_window, &stop_send).await;
    });
}

fn record_subscribe_metrics_thread(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    subscribe_manager: Arc<SubscribeManager>,
    time_window: u64,
    stop_send: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let record_func = async || -> ResultCommonError {
            let now: u64 = now_second();

            for (_, sub) in subscribe_manager.list_subscribe() {
                // send success
                let num = get_subscribe_messages_sent(&sub.client_id, &sub.path, true);
                let pre_num = metrics_cache_manager
                    .get_subscribe_send_pre_total(&sub.client_id, &sub.path, true, num)
                    .await?;
                metrics_cache_manager.record_subscribe_send_num(
                    &sub.client_id,
                    &sub.path,
                    true,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                )?;

                // send failure
                let num = get_subscribe_messages_sent(&sub.client_id, &sub.path, false);
                let pre_num = metrics_cache_manager
                    .get_subscribe_send_pre_total(&sub.client_id, &sub.path, false, num)
                    .await?;
                metrics_cache_manager.record_subscribe_send_num(
                    &sub.client_id,
                    &sub.path,
                    false,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                )?;
            }

            for (_, sub) in subscribe_manager.exclusive_push_list() {
                // send success
                let num = get_subscribe_topic_messages_sent(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true,
                );

                let pre_num = metrics_cache_manager
                    .get_subscribe_topic_send_pre_total(
                        &sub.client_id,
                        &sub.sub_path,
                        &sub.topic_name,
                        true,
                        num,
                    )
                    .await?;

                metrics_cache_manager.record_subscribe_topic_send_num(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                )?;

                // send failure
                let num = get_subscribe_topic_messages_sent(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false,
                );

                let pre_num = metrics_cache_manager
                    .get_subscribe_topic_send_pre_total(
                        &sub.client_id,
                        &sub.sub_path,
                        &sub.topic_name,
                        false,
                        num,
                    )
                    .await?;
                metrics_cache_manager.record_subscribe_topic_send_num(
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false,
                    now,
                    num,
                    calc_value(num, pre_num, time_window),
                )?;
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
    metrics_cache_manager: Arc<MQTTMetricsCache>,
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
        cache_manager.clone(),
        time_window,
        stop_send.clone(),
    );

    record_subscribe_metrics_thread(
        metrics_cache_manager.clone(),
        subscribe_manager.clone(),
        time_window,
        stop_send.clone(),
    );

    record_session_metrics_thread(
        metrics_cache_manager.clone(),
        cache_manager.clone(),
        time_window,
        stop_send.clone(),
    );
}

fn calc_value(max_value: u64, min_value: u64, time_window: u64) -> u64 {
    if time_window == 0 {
        return 0;
    }

    let diff = (max_value - min_value) as f64;
    let window = time_window as f64;
    let result = diff / window;

    result.round() as u64
}

#[cfg(test)]
mod test {
    use crate::common::metrics_cache::calc_value;

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
}
