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

    pub fn get_leader_num_by_node(&self) -> Result<HashMap<String, MqttGroupLeader>, CommonError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_storage() -> MqttGroupLeaderStorage {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        MqttGroupLeaderStorage::new(test_rocksdb_instance())
    }
}
