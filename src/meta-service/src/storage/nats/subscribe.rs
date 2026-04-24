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
use metadata_struct::nats::subscribe::NatsSubscribe;
use rocksdb_engine::keys::meta::{
    storage_key_nats_subscribe, storage_key_nats_subscribe_broker_prefix,
    storage_key_nats_subscribe_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_data::{
    engine_delete_by_meta_data, engine_get_by_meta_data, engine_prefix_list_by_meta_data,
    engine_save_by_meta_data,
};
use std::sync::Arc;

pub struct NatsSubscribeStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl NatsSubscribeStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        NatsSubscribeStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, subscribe: &NatsSubscribe) -> Result<(), CommonError> {
        let key =
            storage_key_nats_subscribe(subscribe.broker_id, subscribe.connect_id, &subscribe.sid);
        engine_save_by_meta_data(&self.rocksdb_engine_handler, &key, subscribe)
    }

    pub fn get(
        &self,
        broker_id: u64,
        connect_id: u64,
        sid: &str,
    ) -> Result<Option<NatsSubscribe>, CommonError> {
        let key = storage_key_nats_subscribe(broker_id, connect_id, sid);
        Ok(
            engine_get_by_meta_data::<NatsSubscribe>(&self.rocksdb_engine_handler, &key)?
                .map(|data| data.data),
        )
    }

    pub fn list(&self) -> Result<Vec<NatsSubscribe>, CommonError> {
        let prefix = storage_key_nats_subscribe_prefix();
        let data = engine_prefix_list_by_meta_data::<NatsSubscribe>(
            &self.rocksdb_engine_handler,
            &prefix,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_broker(&self, broker_id: u64) -> Result<Vec<NatsSubscribe>, CommonError> {
        let prefix = storage_key_nats_subscribe_broker_prefix(broker_id);
        let data = engine_prefix_list_by_meta_data::<NatsSubscribe>(
            &self.rocksdb_engine_handler,
            &prefix,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn delete(&self, broker_id: u64, connect_id: u64, sid: &str) -> Result<(), CommonError> {
        let key = storage_key_nats_subscribe(broker_id, connect_id, sid);
        engine_delete_by_meta_data(&self.rocksdb_engine_handler, &key)
    }
}
