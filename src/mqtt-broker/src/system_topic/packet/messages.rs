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

use crate::core::cache::MQTTCacheManager;
use crate::system_topic::report_system_data;
use common_metrics::mqtt::publish::{
    record_messages_dropped_no_subscribers_get, record_mqtt_messages_received_get,
    record_mqtt_messages_sent_get,
};
use common_metrics::mqtt::statistics::record_mqtt_retained_get;
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

// MQTT Message Received and Sent
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RECEIVED: &str =
    "$SYS/brokers/metrics/messages/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_SENT: &str =
    "$SYS/brokers/metrics/messages/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_EXPIRED: &str =
    "$SYS/brokers/metrics/messages/expired";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RETAINED: &str =
    "$SYS/brokers/metrics/messages/retained";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_DROPPED: &str =
    "$SYS/brokers/metrics/messages/dropped";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_FORWARD: &str =
    "$SYS/brokers/metrics/messages/forward";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_RECEIVED: &str =
    "$SYS/brokers/metrics/messages/qos0/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_SENT: &str =
    "$SYS/brokers/metrics/messages/qos0/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_RECEIVED: &str =
    "$SYS/brokers/metrics/messages/qos1/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_SENT: &str =
    "$SYS/brokers/metrics/messages/qos1/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_RECEIVED: &str =
    "$SYS/brokers/metrics/messages/qos2/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_SENT: &str =
    "$SYS/brokers/metrics/messages/qos2/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_EXPIRED: &str =
    "$SYS/brokers/metrics/messages/qos2/expired";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_DROPPED: &str =
    "$SYS/brokers/metrics/messages/qos2/dropped";

pub(crate) async fn report_broker_metrics_messages(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    let received = record_mqtt_messages_received_get();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RECEIVED,
        || async move { received.to_string() },
    )
    .await;

    let sent = record_mqtt_messages_sent_get();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_SENT,
        || async move { sent.to_string() },
    )
    .await;

    // expired: not currently tracked separately, report 0
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_EXPIRED,
        || async { "0".to_string() },
    )
    .await;

    let retained = record_mqtt_retained_get();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RETAINED,
        || async move { retained.to_string() },
    )
    .await;

    let dropped = record_messages_dropped_no_subscribers_get();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_DROPPED,
        || async move { dropped.to_string() },
    )
    .await;

    // forward: not tracked separately
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_FORWARD,
        || async { "0".to_string() },
    )
    .await;

    // Per-QoS counters: not tracked at this granularity, report 0
    for topic_const in [
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_RECEIVED,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_SENT,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_RECEIVED,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_SENT,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_RECEIVED,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_SENT,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_EXPIRED,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_DROPPED,
    ] {
        report_system_data(
            client_pool,
            metadata_cache,
            storage_driver_manager,
            topic_const,
            || async { "0".to_string() },
        )
        .await;
    }
}
