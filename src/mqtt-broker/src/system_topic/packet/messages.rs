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
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

// MQTT Message Received and Sent
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_EXPIRED: &str =
    "$SYS/brokers/${node}/metrics/messages/expired";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RETAINED: &str =
    "$SYS/brokers/${node}/metrics/messages/retained";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_DROPPED: &str =
    "$SYS/brokers/${node}/metrics/messages/dropped";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_FORWARD: &str =
    "$SYS/brokers/${node}/metrics/messages/forward";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos0/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/qos0/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos1/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/qos1/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_RECEIVED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/received";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_SENT: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/sent";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_EXPIRED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/expired";
pub(crate) const SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_DROPPED: &str =
    "$SYS/brokers/${node}/metrics/messages/qos2/dropped";

pub(crate) async fn report_broker_metrics_messages(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) {
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_messages_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_messages_sent().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_EXPIRED,
        || async {
            "".to_string()
            // metadata_cache.get_messages_expired().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_RETAINED,
        || async {
            "".to_string()
            // metadata_cache.get_messages_retained().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_DROPPED,
        || async {
            "".to_string()
            // metadata_cache.get_messages_dropped().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_FORWARD,
        || async {
            "".to_string()
            // metadata_cache.get_messages_forward().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_messages_qos0_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS0_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_messages_qos0_sent().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_messages_qos1_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS1_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_messages_qos1_sent().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_RECEIVED,
        || async {
            "".to_string()
            // metadata_cache.get_messages_qos2_received().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_SENT,
        || async {
            "".to_string()
            // metadata_cache.get_messages_qos2_sent().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_EXPIRED,
        || async {
            "".to_string()
            // metadata_cache.get_messages_qos2_expired().to_string()
        },
    )
    .await;

    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_METRICS_MESSAGES_QOS2_DROPPED,
        || async {
            "".to_string()
            // metadata_cache.get_messages_qos2_dropped().to_string()
        },
    )
    .await;
}
