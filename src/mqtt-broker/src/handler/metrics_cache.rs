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

use crate::{
    bridge::manager::ConnectorManager, handler::cache::MQTTCacheManager,
    subscribe::manager::SubscribeManager,
};
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use common_metrics::mqtt::connector::{
    get_connector_messages_sent_failure, get_connector_messages_sent_failure_total,
    get_connector_messages_sent_success, get_connector_messages_sent_success_total,
};
use common_metrics::mqtt::publish::{
    record_messages_dropped_no_subscribers_get, record_mqtt_messages_received_get,
    record_mqtt_messages_sent_get,
};
use common_metrics::mqtt::session::{get_session_messages_in, get_session_messages_out};
use common_metrics::mqtt::statistics::{
    record_mqtt_connections_set, record_mqtt_sessions_set, record_mqtt_subscribers_set,
    record_mqtt_subscriptions_shared_set, record_mqtt_topics_set,
};
use common_metrics::mqtt::subscribe::get_subscribe_messages_sent;
use common_metrics::mqtt::topic::{get_topic_messages_sent, get_topic_messages_written};
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::metrics::mqtt::MQTTMetricsCache;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

macro_rules! record_cumulative_metric {
    ($cache:expr, $record_fn:ident, $get_pre_fn:ident, $now:expr, $num:expr, $time_window:expr) => {{
        let pre_num = $cache.$get_pre_fn().await?;
        $cache
            .$record_fn($now, $num, calc_value($num, pre_num, $time_window))
            .await?;
    }};
    ($cache:expr, $record_fn:ident, $get_pre_fn:ident, $now:expr, $num:expr, $time_window:expr, $($dim:expr),+) => {{
        let pre_num = $cache.$get_pre_fn($($dim,)+ $num).await?;
        $cache
            .$record_fn($($dim,)+ $now, $num, calc_value($num, pre_num, $time_window))?;
    }};
}

macro_rules! record_metric_safe {
    ($ctx:expr, $body:block) => {
        match (async { $body }).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to record metric in {}: {}", $ctx, e);
            }
        }
    };
}

async fn record_basic_metrics(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    cache_manager: Arc<MQTTCacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    time_window: u64,
) -> ResultCommonError {
    let now: u64 = now_second();

    // connection num
    metrics_cache_manager
        .record_connection_num(now, connection_manager.connections.len() as u64)?;

    // topic num
    metrics_cache_manager
        .record_topic_num(now, cache_manager.broker_cache.topic_list.len() as u64)?;

    // subscribe num
    metrics_cache_manager
        .record_subscribe_num(now, subscribe_manager.subscribe_list.len() as u64)?;

    // record metrics
    record_mqtt_connections_set(connection_manager.connections.len() as i64);
    record_mqtt_sessions_set(cache_manager.session_info.len() as i64);
    record_mqtt_topics_set(cache_manager.broker_cache.topic_list.len() as i64);
    record_mqtt_subscribers_set(subscribe_manager.subscribe_list.len() as i64);
    record_mqtt_subscriptions_shared_set(0);

    // message in
    let num = record_mqtt_messages_received_get();
    record_cumulative_metric!(
        metrics_cache_manager,
        record_message_in_num,
        get_pre_message_in,
        now,
        num,
        time_window
    );

    // message out
    let num = record_mqtt_messages_sent_get();
    record_cumulative_metric!(
        metrics_cache_manager,
        record_message_out_num,
        get_pre_message_out,
        now,
        num,
        time_window
    );

    // message drop
    let num = record_messages_dropped_no_subscribers_get();
    record_cumulative_metric!(
        metrics_cache_manager,
        record_message_drop_num,
        get_pre_message_drop,
        now,
        num,
        time_window
    );

    Ok(())
}

async fn record_topic_metrics(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    cache_manager: Arc<MQTTCacheManager>,
    time_window: u64,
) -> ResultCommonError {
    let now: u64 = now_second();

    for topic in cache_manager.broker_cache.get_all_topic_name() {
        record_metric_safe!(format!("topic {}", topic), {
            // topic in
            let num = get_topic_messages_written(&topic);
            record_cumulative_metric!(
                metrics_cache_manager,
                record_topic_in_num,
                get_topic_in_pre_total,
                now,
                num,
                time_window,
                &topic
            );

            // topic out
            let num = get_topic_messages_sent(&topic);
            record_cumulative_metric!(
                metrics_cache_manager,
                record_topic_out_num,
                get_topic_out_pre_total,
                now,
                num,
                time_window,
                &topic
            );
            Ok::<(), common_base::error::common::CommonError>(())
        });
    }

    Ok(())
}

async fn record_session_metrics(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    cache_manager: Arc<MQTTCacheManager>,
    time_window: u64,
) -> ResultCommonError {
    let now: u64 = now_second();

    for client_id in cache_manager.get_session_client_id_list() {
        record_metric_safe!(format!("session {}", client_id), {
            // session in
            let num = get_session_messages_in(&client_id);
            record_cumulative_metric!(
                metrics_cache_manager,
                record_session_in_num,
                get_session_in_pre_total,
                now,
                num,
                time_window,
                &client_id
            );

            // session out
            let num = get_session_messages_out(&client_id);
            record_cumulative_metric!(
                metrics_cache_manager,
                record_session_out_num,
                get_session_out_pre_total,
                now,
                num,
                time_window,
                &client_id
            );
            Ok::<(), common_base::error::common::CommonError>(())
        });
    }

    Ok(())
}

async fn record_connector_metrics(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    connector_manager: Arc<ConnectorManager>,
    time_window: u64,
) -> ResultCommonError {
    let now: u64 = now_second();

    for connector in connector_manager.get_all_connector() {
        record_metric_safe!(format!("connector {}", connector.connector_name), {
            // success messages
            let num = get_connector_messages_sent_success(&connector.connector_name);
            record_cumulative_metric!(
                metrics_cache_manager,
                record_connector_success_num,
                get_connector_success_pre_total,
                now,
                num,
                time_window,
                &connector.connector_name
            );

            // failure messages
            let num = get_connector_messages_sent_failure(&connector.connector_name);
            record_cumulative_metric!(
                metrics_cache_manager,
                record_connector_failure_num,
                get_connector_failure_pre_total,
                now,
                num,
                time_window,
                &connector.connector_name
            );
            Ok::<(), common_base::error::common::CommonError>(())
        });
    }

    // total success messages
    let num = get_connector_messages_sent_success_total();
    record_cumulative_metric!(
        metrics_cache_manager,
        record_connector_success_total_num,
        get_connector_success_total_pre,
        now,
        num,
        time_window
    );

    // total failure messages
    let num = get_connector_messages_sent_failure_total();
    record_cumulative_metric!(
        metrics_cache_manager,
        record_connector_failure_total_num,
        get_connector_failure_total_pre,
        now,
        num,
        time_window
    );

    Ok(())
}

async fn record_subscribe_metrics(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    subscribe_manager: Arc<SubscribeManager>,
    time_window: u64,
) -> ResultCommonError {
    let now: u64 = now_second();

    for raw in subscribe_manager.subscribe_list.iter() {
        let sub = raw.value();
        record_metric_safe!(format!("subscribe {}:{}", sub.client_id, sub.path), {
            // send success
            let num = get_subscribe_messages_sent(&sub.client_id, &sub.path, true);
            record_cumulative_metric!(
                metrics_cache_manager,
                record_subscribe_send_num,
                get_subscribe_send_pre_total,
                now,
                num,
                time_window,
                &sub.client_id,
                &sub.path,
                true
            );

            // send failure
            let num = get_subscribe_messages_sent(&sub.client_id, &sub.path, false);
            record_cumulative_metric!(
                metrics_cache_manager,
                record_subscribe_send_num,
                get_subscribe_send_pre_total,
                now,
                num,
                time_window,
                &sub.client_id,
                &sub.path,
                false
            );
            Ok::<(), common_base::error::common::CommonError>(())
        });
    }

    // for row1 in subscribe_manager
    //     .directly_sub_manager
    //     .directly_client_id_push
    //     .iter()
    // {
    //     for raw in row1.value().data_list.iter() {
    //         for sub in raw.value().iter() {
    //             record_metric_safe!(
    //                 format!(
    //                     "exclusive {}:{}:{}",
    //                     sub.client_id, sub.sub_path, sub.topic_name
    //                 ),
    //                 {
    //                     // send success
    //                     let num = get_subscribe_topic_messages_sent(
    //                         &sub.client_id,
    //                         &sub.sub_path,
    //                         &sub.topic_name,
    //                         true,
    //                     );
    //                     record_cumulative_metric!(
    //                         metrics_cache_manager,
    //                         record_subscribe_topic_send_num,
    //                         get_subscribe_topic_send_pre_total,
    //                         now,
    //                         num,
    //                         time_window,
    //                         &sub.client_id,
    //                         &sub.sub_path,
    //                         &sub.topic_name,
    //                         true
    //                     );

    //                     // send failure
    //                     let num = get_subscribe_topic_messages_sent(
    //                         &sub.client_id,
    //                         &sub.sub_path,
    //                         &sub.topic_name,
    //                         false,
    //                     );
    //                     record_cumulative_metric!(
    //                         metrics_cache_manager,
    //                         record_subscribe_topic_send_num,
    //                         get_subscribe_topic_send_pre_total,
    //                         now,
    //                         num,
    //                         time_window,
    //                         &sub.client_id,
    //                         &sub.sub_path,
    //                         &sub.topic_name,
    //                         false
    //                     );
    //                     Ok::<(), common_base::error::common::CommonError>(())
    //                 }
    //             );
    //         }
    //     }
    // }

    // for (_, share_data) in subscribe_manager.share_push_list() {
    //     for sub_entry in share_data.sub_list.iter() {
    //         let client_id = sub_entry.key();
    //         let subscriber = sub_entry.value();
    //         record_metric_safe!(
    //             format!(
    //                 "shared {}:{}:{}",
    //                 client_id, subscriber.sub_path, subscriber.topic_name
    //             ),
    //             {
    //                 // send success
    //                 let num = get_subscribe_topic_messages_sent(
    //                     client_id,
    //                     &subscriber.sub_path,
    //                     &subscriber.topic_name,
    //                     true,
    //                 );
    //                 record_cumulative_metric!(
    //                     metrics_cache_manager,
    //                     record_subscribe_topic_send_num,
    //                     get_subscribe_topic_send_pre_total,
    //                     now,
    //                     num,
    //                     time_window,
    //                     client_id,
    //                     &subscriber.sub_path,
    //                     &subscriber.topic_name,
    //                     true
    //                 );

    //                 // send failure
    //                 let num = get_subscribe_topic_messages_sent(
    //                     client_id,
    //                     &subscriber.sub_path,
    //                     &subscriber.topic_name,
    //                     false,
    //                 );
    //                 record_cumulative_metric!(
    //                     metrics_cache_manager,
    //                     record_subscribe_topic_send_num,
    //                     get_subscribe_topic_send_pre_total,
    //                     now,
    //                     num,
    //                     time_window,
    //                     client_id,
    //                     &subscriber.sub_path,
    //                     &subscriber.topic_name,
    //                     false
    //                 );
    //                 Ok::<(), common_base::error::common::CommonError>(())
    //             }
    //         );
    //     }
    // }

    Ok(())
}

pub fn metrics_record_thread(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    cache_manager: Arc<MQTTCacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    connector_manager: Arc<ConnectorManager>,
    time_window: u64,
    stop_send: broadcast::Sender<bool>,
) {
    info!("Metrics record thread start successfully");

    // Basic metrics thread
    {
        let metrics_cache_manager = metrics_cache_manager.clone();
        let cache_manager = cache_manager.clone();
        let subscribe_manager = subscribe_manager.clone();
        let connection_manager = connection_manager;
        let stop_send = stop_send.clone();
        tokio::spawn(Box::pin(async move {
            let record_func = async || {
                record_basic_metrics(
                    metrics_cache_manager.clone(),
                    cache_manager.clone(),
                    subscribe_manager.clone(),
                    connection_manager.clone(),
                    time_window,
                )
                .await
            };
            loop_select_ticket(record_func, time_window * 1000, &stop_send).await;
        }));
    }

    // Topic metrics thread
    {
        let metrics_cache_manager = metrics_cache_manager.clone();
        let cache_manager = cache_manager.clone();
        let stop_send = stop_send.clone();
        tokio::spawn(Box::pin(async move {
            let record_func = async || {
                record_topic_metrics(
                    metrics_cache_manager.clone(),
                    cache_manager.clone(),
                    time_window,
                )
                .await
            };
            loop_select_ticket(record_func, time_window * 1000, &stop_send).await;
        }));
    }

    // Subscribe metrics thread
    {
        let metrics_cache_manager = metrics_cache_manager.clone();
        let subscribe_manager = subscribe_manager;
        let stop_send = stop_send.clone();
        tokio::spawn(Box::pin(async move {
            let record_func = async || {
                record_subscribe_metrics(
                    metrics_cache_manager.clone(),
                    subscribe_manager.clone(),
                    time_window,
                )
                .await
            };
            loop_select_ticket(record_func, time_window * 1000, &stop_send).await;
        }));
    }

    // Session metrics thread
    {
        let metrics_cache_manager = metrics_cache_manager.clone();
        let cache_manager = cache_manager;
        let stop_send = stop_send.clone();
        tokio::spawn(Box::pin(async move {
            let record_func = async || {
                record_session_metrics(
                    metrics_cache_manager.clone(),
                    cache_manager.clone(),
                    time_window,
                )
                .await
            };
            loop_select_ticket(record_func, time_window * 1000, &stop_send).await;
        }));
    }

    // Connector metrics thread
    {
        let metrics_cache_manager = metrics_cache_manager;
        let connector_manager = connector_manager;
        let stop_send = stop_send;
        tokio::spawn(Box::pin(async move {
            let record_func = async || {
                record_connector_metrics(
                    metrics_cache_manager.clone(),
                    connector_manager.clone(),
                    time_window,
                )
                .await
            };
            loop_select_ticket(record_func, time_window * 1000, &stop_send).await;
        }));
    }
}

fn calc_value(max_value: u64, min_value: u64, time_window: u64) -> u64 {
    if time_window == 0 {
        return 0;
    }

    if max_value < min_value {
        return 0;
    }

    let diff = max_value.saturating_sub(min_value) as f64;
    let window = time_window as f64;
    let result = diff / window;

    result.round() as u64
}

#[cfg(test)]
mod test {
    use crate::handler::metrics_cache::calc_value;

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

    #[test]
    fn test_calc_value_edge_cases() {
        assert_eq!(calc_value(0, 100, 10), 0);
        assert_eq!(calc_value(50, 100, 10), 0);
        assert_eq!(calc_value(u64::MAX, u64::MAX - 100, 10), 10);
        assert_eq!(calc_value(100, 200, 10), 0);
    }
}
