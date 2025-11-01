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

use crate::{
    memory::MemoryStorageAdapter,
    message_expire::{message_expire_thread, MessageExpireConfig},
    mysql::MySQLStorageAdapter,
    rocksdb::RocksDBStorageAdapter,
    storage::ArcStorageAdapter,
    StorageType,
};
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::{str::FromStr, sync::Arc};
use third_driver::mysql::build_mysql_conn_pool;

pub fn build_message_storage_driver(
    db: Arc<RocksDBEngine>,
) -> Result<ArcStorageAdapter, CommonError> {
    let conf = broker_config();
    let storage_type = StorageType::from_str(conf.mqtt_message_storage.storage_type.as_str())
        .expect("Storage type not supported");

    let storage: ArcStorageAdapter = match storage_type {
        StorageType::Memory => Arc::new(MemoryStorageAdapter::new()),

        StorageType::Mysql => {
            let pool = build_mysql_conn_pool(&conf.mqtt_message_storage.mysql_addr)?;
            Arc::new(MySQLStorageAdapter::new(pool.clone())?)
        }

        StorageType::RocksDB => Arc::new(RocksDBStorageAdapter::new(db)),

        _ => {
            return Err(CommonError::UnavailableStorageType);
        }
    };

    message_expire_thread(storage.clone(), MessageExpireConfig::default());
    Ok(storage)
}
