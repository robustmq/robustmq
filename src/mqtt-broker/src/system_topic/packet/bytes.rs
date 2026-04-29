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
use common_metrics::mqtt::packets::{
    record_mqtt_total_bytes_received_get, record_mqtt_total_bytes_sent_get,
};
use grpc_clients::pool::ClientPool;
use serde::Serialize;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

use crate::system_topic::SYSTEM_TOPIC_BROKERS_METRICS_BYTES;

/// Aggregated byte counters published as a single JSON payload to
/// `$SYS/brokers/metrics/bytes`.
#[derive(Debug, Serialize)]
pub(crate) struct BrokerBytesMetrics {
    // Total bytes received from clients
    pub received: u64,
    // Total bytes sent to clients
    pub sent: u64,
}

impl BrokerBytesMetrics {
    pub(crate) fn collect() -> Self {
        BrokerBytesMetrics {
            received: record_mqtt_total_bytes_received_get(),
            sent: record_mqtt_total_bytes_sent_get(),
        }
    }
}

pub(crate) async fn report_broker_metrics_bytes(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    let metrics = BrokerBytesMetrics::collect();
    let payload = serde_json::to_string(&metrics).unwrap_or_default();
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_BYTES,
        || async move { payload },
    )
    .await;
}
