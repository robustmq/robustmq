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

use broker_core::cache::BrokerCacheManager;
use common_base::{error::common::CommonError, tools::now_second};
use metadata_struct::mqtt::session::MqttSession;
use node_call::{NodeCallData, NodeCallManager};
use rocksdb_engine::{
    keys::meta::storage_key_mqtt_session,
    rocksdb::RocksDBEngine,
    storage::meta_data::{engine_delete_by_meta_data, engine_get_by_meta_data},
};
use std::sync::Arc;
use tracing::debug;

use crate::{
    handler::lastwill_expire::get_last_will_message, manager::DelayTaskManager, DelayTask,
    DelayTaskData,
};

pub async fn handle_session_expire(
    node_call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    broker_cache: &Arc<BrokerCacheManager>,
    delay_task_manager: &Arc<DelayTaskManager>,
    client_id: &str,
) -> Result<(), CommonError> {
    let mut has_session = false;
    if let Some((session, is_cache)) = get_session(rocksdb_engine_handler, broker_cache, client_id)?
    {
        has_session = true;
        if is_cache {
            broker_cache.delete_session(client_id);
        } else {
            let key = storage_key_mqtt_session(client_id);
            engine_delete_by_meta_data(rocksdb_engine_handler, &key)?;
        }

        let data = NodeCallData::DeleteSession(client_id.to_string());
        node_call_manager.send(data).await?;

        if let Some(delay_interval) = session.last_will_delay_interval {
            let delay_target_time = now_second() + delay_interval;
            delay_task_manager
                .create_task(DelayTask::build_persistent(
                    client_id.to_string(),
                    DelayTaskData::MQTTLastwillExpire(client_id.to_string()),
                    delay_target_time,
                ))
                .await?;
        } else if let Some(will_message) = get_last_will_message(rocksdb_engine_handler, client_id)?
        {
            let data = NodeCallData::SendLastWillMessage(will_message);
            node_call_manager.send(data).await?;
        }
    }

    debug!(
        "Session expire handling completed: client_id={}, session_found={}",
        client_id, has_session
    );
    Ok(())
}

#[allow(clippy::result_large_err)]
fn get_session(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    broker_cache: &Arc<BrokerCacheManager>,
    client_id: &str,
) -> Result<Option<(MqttSession, bool)>, CommonError> {
    if let Some(session) = broker_cache.get_session(client_id) {
        return Ok(Some((session, true)));
    }

    let key = storage_key_mqtt_session(client_id);
    if let Some(session) =
        engine_get_by_meta_data::<MqttSession>(rocksdb_engine_handler, &key)?.map(|data| data.data)
    {
        return Ok(Some((session, false)));
    }
    Ok(None)
}
