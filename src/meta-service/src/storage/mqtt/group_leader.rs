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

use std::{collections::HashMap, sync::Arc};

use common_base::{
    error::{common::CommonError, ResultCommonError},
    tools::now_second,
};
use metadata_struct::mqtt::group_leader::MqttGroupLeader;
use rocksdb_engine::{
    keys::meta::{storage_key_mqtt_group_leader, storage_key_mqtt_group_leader_prefix},
    rocksdb::RocksDBEngine,
    storage::meta_data::{
        engine_delete_by_meta_data, engine_prefix_list_by_meta_data, engine_save_by_meta_data,
    },
};

pub struct MqttGroupLeaderStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttGroupLeaderStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttGroupLeaderStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, group_name: &str, broker_id: u64) -> ResultCommonError {
        let key = storage_key_mqtt_group_leader(group_name);
        let data = MqttGroupLeader {
            group_name: group_name.to_string(),
            broker_id,
            create_time: now_second(),
        };
        engine_save_by_meta_data(&self.rocksdb_engine_handler, &key, data)
    }

    pub fn delete(&self, group_name: &str) -> ResultCommonError {
        let key = storage_key_mqtt_group_leader(group_name);
        engine_delete_by_meta_data(&self.rocksdb_engine_handler, &key)
    }

    pub fn list(&self) -> Result<HashMap<String, MqttGroupLeader>, CommonError> {
        let prefix_key_name = storage_key_mqtt_group_leader_prefix();
        let result = engine_prefix_list_by_meta_data::<MqttGroupLeader>(
            &self.rocksdb_engine_handler,
            &prefix_key_name,
        )?;
        let mut results = HashMap::new();
        for item in result {
            results.insert(item.data.group_name.to_string(), item.data.clone());
        }

        Ok(results)
    }
}
