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
use serde::Serialize;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

use crate::system_topic::SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES;

/// Aggregated message counters published as a single JSON payload to
/// `$SYS/brokers/metrics/messages`.
#[derive(Debug, Serialize)]
pub(crate) struct BrokerMessagesMetrics {
    // Total messages received / sent across all QoS levels
    pub received: u64,
    pub sent: u64,

    // Messages that expired before delivery
    pub expired: u64,

    // Currently stored retained messages (gauge, can decrement hence i64)
    pub retained: i64,

    // Messages dropped due to no matching subscribers
    pub dropped: u64,

    // Messages forwarded to other broker nodes (cluster mode)
    pub forward: u64,

    // Per-QoS received / sent breakdown
    pub qos0_received: u64,
    pub qos0_sent: u64,
    pub qos1_received: u64,
    pub qos1_sent: u64,
    pub qos2_received: u64,
    pub qos2_sent: u64,

    // QoS 2 specific: expired / dropped during handshake
    pub qos2_expired: u64,
    pub qos2_dropped: u64,
}

impl BrokerMessagesMetrics {
    pub(crate) fn collect() -> Self {
        BrokerMessagesMetrics {
            received: record_mqtt_messages_received_get(),
            sent: record_mqtt_messages_sent_get(),
            expired: 0,
            retained: record_mqtt_retained_get(),
            dropped: record_messages_dropped_no_subscribers_get(),
            forward: 0,
            qos0_received: 0,
            qos0_sent: 0,
            qos1_received: 0,
            qos1_sent: 0,
            qos2_received: 0,
            qos2_sent: 0,
            qos2_expired: 0,
            qos2_dropped: 0,
        }
    }
}

pub(crate) async fn report_broker_metrics_messages(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    let metrics = BrokerMessagesMetrics::collect();
    let payload = serde_json::to_string(&metrics).unwrap_or_default();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES,
        || async move { payload },
    )
    .await;
}
