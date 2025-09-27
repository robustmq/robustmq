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
use common_base::tools::now_mills;
use common_metrics::rocksdb::{
    metrics_rocksdb_delete_ms, metrics_rocksdb_exist_ms, metrics_rocksdb_get_ms,
    metrics_rocksdb_list_ms, metrics_rocksdb_save_ms,
};
use rocksdb_engine::engine::{
    rocksdb_engine_delete, rocksdb_engine_exists, rocksdb_engine_get,
    rocksdb_engine_list_by_prefix, rocksdb_engine_save,
};
use rocksdb_engine::warp::StorageDataWrap;
use rocksdb_engine::RocksDBEngine;
use serde::Serialize;
use std::sync::Arc;

use crate::rocksdb::DB_COLUMN_FAMILY_BROKER;

pub fn engine_save_by_broker<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
    value: T,
) -> Result<(), CommonError>
where
    T: Serialize,
{
    let start_time = now_mills();
    let result = rocksdb_engine_save(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_BROKER,
        key_name,
        value,
    );
    let duration = (now_mills() - start_time) as f64;
    metrics_rocksdb_save_ms("broker", duration);
    result
}

pub fn engine_get_by_broker(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<Option<StorageDataWrap>, CommonError> {
    let start_time = now_mills();
    let result = rocksdb_engine_get(rocksdb_engine_handler, DB_COLUMN_FAMILY_BROKER, key_name);
    let duration = (now_mills() - start_time) as f64;
    metrics_rocksdb_get_ms("broker", duration);
    result
}

pub fn engine_exists_by_broker(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<bool, CommonError> {
    let start_time = now_mills();
    let result = rocksdb_engine_exists(rocksdb_engine_handler, DB_COLUMN_FAMILY_BROKER, key_name);
    let duration = (now_mills() - start_time) as f64;
    metrics_rocksdb_exist_ms("broker", duration);
    result
}

pub fn engine_delete_by_broker(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<(), CommonError> {
    let start_time = now_mills();
    let result = rocksdb_engine_delete(rocksdb_engine_handler, DB_COLUMN_FAMILY_BROKER, key_name);
    let duration = (now_mills() - start_time) as f64;
    metrics_rocksdb_delete_ms("broker", duration);
    result
}

pub fn engine_prefix_list_by_broker(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    prefix_key_name: String,
) -> Result<Vec<StorageDataWrap>, CommonError> {
    let start_time = now_mills();
    let result = rocksdb_engine_list_by_prefix(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_BROKER,
        prefix_key_name,
    );
    let duration = (now_mills() - start_time) as f64;
    metrics_rocksdb_list_ms("broker", duration);
    result
}
