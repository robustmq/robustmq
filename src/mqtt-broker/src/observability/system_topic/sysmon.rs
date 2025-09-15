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

use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::MQTTCacheManager;
use crate::handler::system_alarm::SystemAlarmEventMessage;
use crate::observability::system_topic::{replace_topic_name, write_topic_data};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::message::MqttMessage;
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;

// sysmon topic
pub(crate) const SYSTEM_TOPIC_BROKERS_ALARMS_ACTIVATE: &str =
    "$SYS/brokers/${node}/alarms/activate";
pub(crate) const SYSTEM_TOPIC_BROKERS_ALARMS_DEACTIVATE: &str =
    "$SYS/brokers/${node}/alarms/deactivate";

pub async fn st_report_system_alarm_event(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
    message_event: &SystemAlarmEventMessage,
) -> ResultMqttBrokerError {
    let data = serde_json::to_string(message_event)?;
    let mut topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS_ALARMS_ACTIVATE.to_string());

    if !message_event.activated {
        topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS_ALARMS_DEACTIVATE.to_string());
    }

    if let Some(record) = MqttMessage::build_system_topic_message(topic_name.clone(), data) {
        write_topic_data(
            message_storage_adapter,
            metadata_cache,
            client_pool,
            topic_name,
            record,
        )
        .await;
    };

    Ok(())
}
