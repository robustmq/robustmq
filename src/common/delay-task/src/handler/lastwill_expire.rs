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

use common_base::error::common::CommonError;
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use node_call::{NodeCallData, NodeCallManager};
use protocol::broker::broker_mqtt::LastWillMessageItem;
use rocksdb_engine::{
    keys::meta::storage_key_mqtt_last_will,
    rocksdb::RocksDBEngine,
    storage::meta_data::{engine_delete_by_meta_data, engine_get_by_meta_data},
};
use std::sync::Arc;
use tracing::debug;

pub async fn handle_lastwill_expire(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    node_call_manager: &Arc<NodeCallManager>,
    client_id: &str,
) -> Result<(), CommonError> {
    let mut has_lastwill = false;
    if let Some(lastwill) = get_last_will_message(rocksdb_engine_handler, client_id)? {
        has_lastwill = true;
        node_call_manager
            .send(NodeCallData::SendLastWillMessage(lastwill))
            .await?;

        let key = storage_key_mqtt_last_will(client_id);
        engine_delete_by_meta_data(rocksdb_engine_handler, &key)?
    }
    debug!(
        "Lastwill expire handling completed: client_id={}, lastwill_found={}",
        client_id, has_lastwill
    );
    Ok(())
}

#[allow(clippy::result_large_err)]
pub(crate) fn get_last_will_message(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    client_id: &str,
) -> Result<Option<LastWillMessageItem>, CommonError> {
    let key = storage_key_mqtt_last_will(client_id);
    if let Some(lastwill) =
        engine_get_by_meta_data::<MqttLastWillData>(rocksdb_engine_handler, &key)?
            .map(|data| data.data)
    {
        let data = lastwill.encode()?;
        let will_message = LastWillMessageItem {
            client_id: client_id.to_string(),
            last_will_message: data,
        };
        return Ok(Some(will_message));
    }
    Ok(None)
}
