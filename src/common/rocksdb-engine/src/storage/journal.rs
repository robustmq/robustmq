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
    rocksdb::RocksDBEngine,
    storage::{
        base::{
            engine_delete, engine_delete_prefix, engine_exists, engine_get, engine_list_by_model,
            engine_prefix_list, engine_save,
        },
        family::DB_COLUMN_FAMILY_JOURNAL,
    },
    warp::StorageDataWrap,
};
use common_base::{error::common::CommonError, utils::serialize};
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

pub fn engine_save_by_journal<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    key_name: &str,
    value: T,
) -> Result<(), CommonError>
where
    T: Serialize,
{
    engine_save(
        rocksdb_engine_handler,
        column_family,
        DB_COLUMN_FAMILY_JOURNAL,
        key_name,
        value,
    )
}

pub fn engine_get_by_journal<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    key_name: &str,
) -> Result<Option<StorageDataWrap<T>>, CommonError>
where
    T: DeserializeOwned,
{
    engine_get(
        rocksdb_engine_handler,
        column_family,
        DB_COLUMN_FAMILY_JOURNAL,
        key_name,
    )
}

pub fn engine_delete_by_journal(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    key_name: &str,
) -> Result<(), CommonError> {
    engine_delete(
        rocksdb_engine_handler,
        column_family,
        DB_COLUMN_FAMILY_JOURNAL,
        key_name,
    )
}

pub fn engine_delete_prefix_by_journal(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    prefix_key: &str,
) -> Result<(), CommonError> {
    engine_delete_prefix(
        rocksdb_engine_handler,
        column_family,
        DB_COLUMN_FAMILY_JOURNAL,
        prefix_key,
    )
}

pub fn engine_exists_by_journal(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    key_name: &str,
) -> Result<bool, CommonError> {
    engine_exists(
        rocksdb_engine_handler,
        column_family,
        DB_COLUMN_FAMILY_JOURNAL,
        key_name,
    )
}

pub fn engine_list_by_prefix_by_journal<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    prefix_key_name: &str,
) -> Result<Vec<StorageDataWrap<T>>, CommonError>
where
    T: DeserializeOwned,
{
    engine_prefix_list(
        rocksdb_engine_handler,
        column_family,
        DB_COLUMN_FAMILY_JOURNAL,
        prefix_key_name,
    )
}

pub fn engine_list_by_mode_by_journal<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    mode: &rocksdb::IteratorMode,
) -> Result<DashMap<String, StorageDataWrap<T>>, CommonError>
where
    T: DeserializeOwned,
{
    engine_list_by_model(
        rocksdb_engine_handler,
        column_family,
        DB_COLUMN_FAMILY_JOURNAL,
        mode,
    )
}

pub fn engine_list_by_prefix_to_map_by_journal<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    prefix_key_name: &str,
) -> Result<DashMap<String, StorageDataWrap<T>>, CommonError>
where
    T: DeserializeOwned,
{
    use common_base::tools::now_mills;
    use common_metrics::rocksdb::metrics_rocksdb_list_ms;

    let start_time = now_mills();

    let cf = rocksdb_engine_handler
        .cf_handle(column_family)
        .ok_or_else(|| CommonError::RocksDBFamilyNotAvailable(column_family.to_string()))?;

    let raw = rocksdb_engine_handler.read_prefix(cf, prefix_key_name)?;
    let results = DashMap::with_capacity(raw.len().min(64));

    for (key, v) in raw {
        match serialize::deserialize::<StorageDataWrap<T>>(v.as_ref()) {
            Ok(v) => {
                results.insert(key.clone(), v);
            }
            Err(_e) => {
                // Silently skip deserialization errors
                continue;
            }
        }
    }

    let duration = (now_mills() - start_time) as f64;
    metrics_rocksdb_list_ms(DB_COLUMN_FAMILY_JOURNAL, duration);

    Ok(results)
}
