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
use crate::core::system_alarm::SystemAlarmEventMessage;
use crate::system_topic::{
    report_system_data, SYSTEM_TOPIC_BROKERS_ALARMS_ALERT, SYSTEM_TOPIC_BROKERS_ALARMS_CLEAR,
};
use common_base::error::ResultCommonError;
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

pub async fn st_report_system_alarm_alert(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    message_event: &SystemAlarmEventMessage,
) -> ResultCommonError {
    let data = serde_json::to_string(message_event)?;
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_ALARMS_ALERT,
        || async move { data },
    )
    .await;
    Ok(())
}

pub async fn st_report_system_alarm_clear(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    message_event: &SystemAlarmEventMessage,
) -> ResultCommonError {
    let data = serde_json::to_string(message_event)?;
    report_system_data(
        client_pool,
        metadata_cache,
        storage_driver_manager,
        SYSTEM_TOPIC_BROKERS_ALARMS_CLEAR,
        || async move { data },
    )
    .await;
    Ok(())
}
