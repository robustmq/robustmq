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

use crate::{core::cache::MQTTCacheManager, subscribe::manager::SubscribeManager};
use common_base::error::ResultCommonError;
use common_base::task::{TaskKind, TaskSupervisor};
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
    record_mqtt_subscriptions_exclusive_set, record_mqtt_subscriptions_shared_group_set,
    record_mqtt_subscriptions_shared_set, record_mqtt_topics_set,
};
use common_metrics::mqtt::subscribe::{
    get_subscribe_messages_sent, get_subscribe_topic_messages_sent,
};
use common_metrics::mqtt::topic::{get_topic_messages_sent, get_topic_messages_written};
use connector::manager::ConnectorManager;
use network_server::common::connection_manager::ConnectionManager;
use rocksdb_engine::metrics::mqtt::MQTTMetricsCache;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::error;

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
    metrics_cache_manager.record_topic_num(now, cache_manager.node_cache.topic_count() as u64)?;

    // subscribe num
    metrics_cache_manager.record_subscribe_num(now, subscribe_manager.subscribe_count() as u64)?;

    // record metrics
    record_mqtt_connections_set(connection_manager.connections.len() as i64);
    record_mqtt_sessions_set(cache_manager.session_count() as i64);
    record_mqtt_topics_set(cache_manager.node_cache.topic_count() as i64);

    record_mqtt_subscribers_set(subscribe_manager.subscribe_count() as i64);
    record_mqtt_subscriptions_exclusive_set(subscribe_manager.directly_push.sub_len() as i64);

    record_mqtt_subscriptions_shared_set(subscribe_manager.share_sub_len() as i64);
    record_mqtt_subscriptions_shared_group_set(subscribe_manager.share_group_count() as i64);

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

    // Collect (tenant, topic) pairs first to release all DashMap shard locks before .await
    let pairs: Vec<(String, String)> = cache_manager
        .node_cache
        .topic_list
        .iter()
        .map(|e| (e.value().tenant.clone(), e.value().topic_name.clone()))
        .collect();

    for (tenant, topic) in pairs {
        record_metric_safe!(format!("topic {}/{}", tenant, topic), {
            // topic in
            let num = get_topic_messages_written(&tenant, &topic);
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
            let num = get_topic_messages_sent(&tenant, &topic);
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

    // Collect (tenant, client_id) pairs first to release all DashMap shard locks before .await
    let pairs: Vec<(String, String)> = cache_manager
        .session_info
        .iter()
        .map(|e| (e.value().tenant.clone(), e.key().clone()))
        .collect();

    for (tenant, client_id) in pairs {
        record_metric_safe!(format!("session {}/{}", tenant, client_id), {
            // session in
            let num = get_session_messages_in(&tenant, &client_id);
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
            let num = get_session_messages_out(&tenant, &client_id);
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
            let connector_type = connector.connector_type.to_string();
            // success messages
            let num = get_connector_messages_sent_success(
                &connector.tenant,
                &connector_type,
                &connector.connector_name,
            );
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
            let num = get_connector_messages_sent_failure(
                &connector.tenant,
                &connector_type,
                &connector.connector_name,
            );
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

struct SubMetricKey {
    tenant: String,
    client_id: String,
    path: String,
}

struct SubTopicMetricKey {
    tenant: String,
    client_id: String,
    sub_path: String,
    topic_name: String,
}

async fn record_subscribe_metrics(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    subscribe_manager: Arc<SubscribeManager>,
    time_window: u64,
) -> ResultCommonError {
    let now: u64 = now_second();

    // Collect all keys first to release DashMap shard locks before any .await
    let sub_keys: Vec<SubMetricKey> = subscribe_manager
        .subscribe_list
        .iter()
        .flat_map(|tenant_entry| {
            tenant_entry
                .value()
                .iter()
                .map(|raw| SubMetricKey {
                    tenant: raw.value().tenant.clone(),
                    client_id: raw.value().client_id.clone(),
                    path: raw.value().path.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let exclusive_keys: Vec<SubTopicMetricKey> = subscribe_manager
        .directly_push
        .buckets_data_list
        .iter()
        .flat_map(|row1| {
            row1.value()
                .iter()
                .map(|sub| SubTopicMetricKey {
                    tenant: sub.tenant.clone(),
                    client_id: sub.client_id.clone(),
                    sub_path: sub.sub_path.clone(),
                    topic_name: sub.topic_name.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let shared_keys: Vec<SubTopicMetricKey> = subscribe_manager
        .share_push
        .iter()
        .flat_map(|tenant_entry| {
            tenant_entry
                .value()
                .iter()
                .flat_map(|share_data| {
                    share_data
                        .value()
                        .buckets_data_list
                        .iter()
                        .flat_map(|raw| {
                            raw.iter()
                                .map(|sub_entry| SubTopicMetricKey {
                                    tenant: sub_entry.value().tenant.clone(),
                                    client_id: sub_entry.value().client_id.clone(),
                                    sub_path: sub_entry.value().sub_path.clone(),
                                    topic_name: sub_entry.value().topic_name.clone(),
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // All locks released — now safe to .await
    for sub in sub_keys {
        record_metric_safe!(
            format!("subscribe {}/{}:{}", sub.tenant, sub.client_id, sub.path),
            {
                let num = get_subscribe_messages_sent(&sub.tenant, &sub.client_id, &sub.path, true);
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

                let num =
                    get_subscribe_messages_sent(&sub.tenant, &sub.client_id, &sub.path, false);
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
            }
        );
    }

    for sub in exclusive_keys {
        record_metric_safe!(
            format!(
                "exclusive {}/{}:{}:{}",
                sub.tenant, sub.client_id, sub.sub_path, sub.topic_name
            ),
            {
                let num = get_subscribe_topic_messages_sent(
                    &sub.tenant,
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true,
                );
                record_cumulative_metric!(
                    metrics_cache_manager,
                    record_subscribe_topic_send_num,
                    get_subscribe_topic_send_pre_total,
                    now,
                    num,
                    time_window,
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true
                );

                let num = get_subscribe_topic_messages_sent(
                    &sub.tenant,
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false,
                );
                record_cumulative_metric!(
                    metrics_cache_manager,
                    record_subscribe_topic_send_num,
                    get_subscribe_topic_send_pre_total,
                    now,
                    num,
                    time_window,
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false
                );
                Ok::<(), common_base::error::common::CommonError>(())
            }
        );
    }

    for sub in shared_keys {
        record_metric_safe!(
            format!(
                "shared {}/{}:{}:{}",
                sub.tenant, sub.client_id, sub.sub_path, sub.topic_name
            ),
            {
                let num = get_subscribe_topic_messages_sent(
                    &sub.tenant,
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true,
                );
                record_cumulative_metric!(
                    metrics_cache_manager,
                    record_subscribe_topic_send_num,
                    get_subscribe_topic_send_pre_total,
                    now,
                    num,
                    time_window,
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    true
                );

                let num = get_subscribe_topic_messages_sent(
                    &sub.tenant,
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false,
                );
                record_cumulative_metric!(
                    metrics_cache_manager,
                    record_subscribe_topic_send_num,
                    get_subscribe_topic_send_pre_total,
                    now,
                    num,
                    time_window,
                    &sub.client_id,
                    &sub.sub_path,
                    &sub.topic_name,
                    false
                );
                Ok::<(), common_base::error::common::CommonError>(())
            }
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn metrics_record_thread(
    metrics_cache_manager: Arc<MQTTMetricsCache>,
    cache_manager: Arc<MQTTCacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    connector_manager: Arc<ConnectorManager>,
    time_window: u64,
    stop_send: broadcast::Sender<bool>,
    task_supervisor: Arc<TaskSupervisor>,
) {
    // Basic metrics thread
    let mcm = metrics_cache_manager.clone();
    let cm = cache_manager.clone();
    let sm = subscribe_manager.clone();
    let conm = connection_manager;
    let stop = stop_send.clone();
    task_supervisor.spawn(TaskKind::MQTTMetricsBasic.to_string(), async move {
        let record_func = async || {
            record_basic_metrics(
                mcm.clone(),
                cm.clone(),
                sm.clone(),
                conm.clone(),
                time_window,
            )
            .await
        };
        loop_select_ticket(record_func, time_window * 1000, &stop).await;
    });

    // Topic metrics thread
    let mcm = metrics_cache_manager.clone();
    let cm = cache_manager.clone();
    let stop = stop_send.clone();
    task_supervisor.spawn(TaskKind::MQTTMetricsTopic.to_string(), async move {
        let record_func = async || record_topic_metrics(mcm.clone(), cm.clone(), time_window).await;
        loop_select_ticket(record_func, time_window * 1000, &stop).await;
    });

    // Subscribe metrics thread
    let mcm = metrics_cache_manager.clone();
    let sm = subscribe_manager;
    let stop = stop_send.clone();
    task_supervisor.spawn(TaskKind::MQTTMetricsSubscribe.to_string(), async move {
        let record_func =
            async || record_subscribe_metrics(mcm.clone(), sm.clone(), time_window).await;
        loop_select_ticket(record_func, time_window * 1000, &stop).await;
    });

    // Session metrics thread
    let mcm = metrics_cache_manager.clone();
    let cm = cache_manager;
    let stop = stop_send.clone();
    task_supervisor.spawn(TaskKind::MQTTMetricsSession.to_string(), async move {
        let record_func =
            async || record_session_metrics(mcm.clone(), cm.clone(), time_window).await;
        loop_select_ticket(record_func, time_window * 1000, &stop).await;
    });

    // Connector metrics thread
    let mcm = metrics_cache_manager;
    let stop = stop_send;
    task_supervisor.spawn(TaskKind::MQTTMetricsConnector.to_string(), async move {
        let record_func = async || {
            record_connector_metrics(mcm.clone(), connector_manager.clone(), time_window).await
        };
        loop_select_ticket(record_func, time_window * 1000, &stop).await;
    });
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
    use crate::core::metrics_cache::calc_value;

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
