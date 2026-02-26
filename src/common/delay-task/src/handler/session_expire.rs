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

use std::sync::Arc;

use common_base::error::common::CommonError;
use metadata_struct::mqtt::session::MqttSession;
use rocksdb_engine::{
    keys::meta::storage_key_mqtt_session, rocksdb::RocksDBEngine,
    storage::meta_data::engine_get_by_meta_data,
};

pub async fn handle_session_expire(
    client_id: &str,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) -> Result<(), CommonError> {
    let key = storage_key_mqtt_session(client_id);
    if let Some(_session) =
        engine_get_by_meta_data::<MqttSession>(rocksdb_engine_handler, &key)?.map(|data| data.data)
    {
    }

    Ok(())
}

pub struct SessionExpireCallManager {}
