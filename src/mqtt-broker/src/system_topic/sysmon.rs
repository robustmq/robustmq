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

use crate::handler::cache::MQTTCacheManager;
use crate::handler::system_alarm::SystemAlarmEventMessage;
use crate::system_topic::{replace_topic_name, write_topic_data};
use common_base::error::ResultCommonError;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::message::MqttMessage;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

// sysmon topic
pub(crate) const SYSTEM_TOPIC_BROKERS_ALARMS_ACTIVATE: &str =
    "$SYS/brokers/${node}/alarms/activate";

pub async fn st_report_system_alarm_event(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    message_event: &SystemAlarmEventMessage,
) -> ResultCommonError {
    let data = serde_json::to_string(message_event)?;
    let topic_name = replace_topic_name(SYSTEM_TOPIC_BROKERS_ALARMS_ACTIVATE.to_string());

    if let Some(record) = MqttMessage::build_system_topic_message(topic_name.clone(), data) {
        let _ = write_topic_data(
            storage_driver_manager,
            metadata_cache,
            client_pool,
            topic_name,
            record,
        )
        .await;
    };

    Ok(())
}
