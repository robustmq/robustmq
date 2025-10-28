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

use crate::handler::error::MqttBrokerError;
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;
use std::collections::HashMap;
use std::str::FromStr;
use storage_adapter::memory::MemoryStorageAdapter;
use storage_adapter::mysql::MySQLStorageAdapter;
use storage_adapter::rocksdb::RocksDBStorageAdapter;
use storage_adapter::storage::{ArcStorageAdapter, StorageAdapter};
use storage_adapter::StorageType;
use third_driver::mysql::build_mysql_conn_pool;

pub fn cluster_name() -> String {
    let conf = broker_config();
    conf.cluster_name.clone()
}

#[derive(Clone)]
pub struct MessageStorage {
    storage_adapter: ArcStorageAdapter,
}

impl MessageStorage {
    pub fn new(storage_adapter: ArcStorageAdapter) -> Self {
        MessageStorage { storage_adapter }
    }

    pub async fn append_topic_message(
        &self,
        topic_name: &str,
        record: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let shard_name = topic_name;
        let namespace = cluster_name();
        let results = self
            .storage_adapter
            .batch_write(namespace, shard_name.to_owned(), record)
            .await?;
        Ok(results)
    }

    pub async fn read_topic_message(
        &self,
        topic_name: &str,
        offset: u64,
        record_num: u64,
    ) -> Result<Vec<Record>, CommonError> {
        let shard_name = topic_name;
        let namespace = cluster_name();
        let mut read_config = ReadConfig::new();
        read_config.max_record_num = record_num;

        let records = self
            .storage_adapter
            .read_by_offset(namespace, shard_name.to_owned(), offset, read_config)
            .await?;
        for raw in records.iter() {
            if !raw.crc32_check() {
                return Err(CommonError::CrcCheckByMessage);
            }
        }
        Ok(records)
    }

    pub async fn get_group_offset(&self, group_id: &str) -> Result<u64, CommonError> {
        let offset_data = self
            .storage_adapter
            .get_offset_by_group(group_id.to_owned())
            .await?;

        if let Some(offset) = offset_data.first() {
            return Ok(offset.offset);
        }
        Ok(0)
    }

    pub async fn commit_group_offset(
        &self,
        group_id: &str,
        topic_name: &str,
        offset: u64,
    ) -> Result<(), CommonError> {
        let shard_name = topic_name;
        let namespace = cluster_name();

        let mut offset_data = HashMap::new();
        offset_data.insert(shard_name.to_owned(), offset);

        self.storage_adapter
            .commit_offset(group_id.to_owned(), namespace, offset_data)
            .await
    }
}

pub fn build_message_storage_driver(
) -> Result<Box<dyn StorageAdapter + Send + Sync>, MqttBrokerError> {
    let conf = broker_config();
    let storage_type = StorageType::from_str(conf.mqtt_message_storage.storage_type.as_str())
        .expect("Storage type not supported");

    let storage: Box<dyn StorageAdapter + Send + Sync> = match storage_type {
        StorageType::Memory => Box::new(MemoryStorageAdapter::new()),

        StorageType::Mysql => {
            let pool = build_mysql_conn_pool(&conf.mqtt_message_storage.mysql_addr)?;
            Box::new(MySQLStorageAdapter::new(pool.clone())?)
        }

        StorageType::RocksDB => Box::new(RocksDBStorageAdapter::new(
            conf.mqtt_message_storage.rocksdb_data_path.as_str(),
            conf.mqtt_message_storage
                .rocksdb_max_open_files
                .unwrap_or(10000),
        )),

        _ => {
            return Err(MqttBrokerError::UnavailableStorageType);
        }
    };

    Ok(storage)
}
