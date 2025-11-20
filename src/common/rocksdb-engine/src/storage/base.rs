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

use crate::{rocksdb::RocksDBEngine, warp::StorageDataWrap};
use common_base::{error::common::CommonError, tools::now_mills, utils};
use common_metrics::rocksdb::{
    metrics_rocksdb_delete_ms, metrics_rocksdb_exist_ms, metrics_rocksdb_get_ms,
    metrics_rocksdb_list_ms, metrics_rocksdb_save_ms,
};
use dashmap::DashMap;
use rocksdb::BoundColumnFamily;
use serde::Serialize;
use std::sync::Arc;

/// Helper function to get column family handle
#[inline]
pub fn get_cf_handle<'a>(
    engine: &'a RocksDBEngine,
    column_family: &str,
) -> Result<Arc<BoundColumnFamily<'a>>, CommonError> {
    engine
        .cf_handle(column_family)
        .ok_or_else(|| CommonError::RocksDBFamilyNotAvailable(column_family.to_string()))
}

/// Macro to simplify metrics collection for RocksDB operations
macro_rules! with_metrics {
    ($source:expr, $metric_fn:expr, $operation:expr) => {{
        let start_time = now_mills();
        let result = $operation;
        let duration = (now_mills() - start_time) as f64;
        $metric_fn($source, duration);
        result
    }};
}

pub fn engine_save<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    source: &str,
    key_name: &str,
    value: T,
) -> Result<(), CommonError>
where
    T: Serialize,
{
    with_metrics!(source, metrics_rocksdb_save_ms, {
        let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
        let wrap = StorageDataWrap::new(value);
        rocksdb_engine_handler.write(cf, key_name, &wrap)?;
        Ok(())
    })
}

pub fn batch_encode_data<T>(value: T) -> Result<Vec<u8>, CommonError>
where
    T: Serialize,
{
    let wrap = StorageDataWrap::new(value);
    utils::serialize::serialize(&wrap)
}

pub fn engine_get<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    source: &str,
    key_name: &str,
) -> Result<Option<StorageDataWrap<T>>, CommonError>
where
    T: serde::de::DeserializeOwned,
{
    with_metrics!(source, metrics_rocksdb_get_ms, {
        let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
        rocksdb_engine_handler.read::<StorageDataWrap<T>>(cf, key_name)
    })
}

pub fn engine_exists(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    source: &str,
    key_name: &str,
) -> Result<bool, CommonError> {
    with_metrics!(source, metrics_rocksdb_exist_ms, {
        let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
        Ok(rocksdb_engine_handler.exist(cf, key_name))
    })
}

pub fn engine_delete(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    source: &str,
    key_name: &str,
) -> Result<(), CommonError> {
    with_metrics!(source, metrics_rocksdb_delete_ms, {
        let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
        rocksdb_engine_handler.delete(cf, key_name)
    })
}

pub fn engine_delete_range(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    source: &str,
    from: Vec<u8>,
    to: Vec<u8>,
) -> Result<(), CommonError> {
    with_metrics!(source, metrics_rocksdb_delete_ms, {
        let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
        rocksdb_engine_handler.delete_range_cf(cf, from, to)
    })
}

pub fn engine_delete_prefix(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    source: &str,
    prefix_key: &str,
) -> Result<(), CommonError> {
    with_metrics!(source, metrics_rocksdb_delete_ms, {
        let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
        rocksdb_engine_handler.delete_prefix(cf, prefix_key)
    })
}

pub fn engine_prefix_list<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    source: &str,
    prefix_key_name: &str,
) -> Result<Vec<StorageDataWrap<T>>, CommonError>
where
    T: serde::de::DeserializeOwned,
{
    use common_base::utils::serialize;

    with_metrics!(source, metrics_rocksdb_list_ms, {
        let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;

        let raw = rocksdb_engine_handler.read_prefix(cf, prefix_key_name)?;
        let mut results = Vec::with_capacity(raw.len().min(64));

        for (_key, v) in raw {
            match serialize::deserialize::<StorageDataWrap<T>>(v.as_ref()) {
                Ok(v) => results.push(v),
                Err(_e) => {
                    // Silently skip deserialization errors
                    continue;
                }
            }
        }
        Ok(results)
    })
}

pub fn engine_list_by_model<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    source: &str,
    mode: &rocksdb::IteratorMode,
) -> Result<DashMap<String, StorageDataWrap<T>>, CommonError>
where
    T: serde::de::DeserializeOwned,
{
    use common_base::utils::serialize;

    with_metrics!(source, metrics_rocksdb_list_ms, {
        let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;

        let raw = rocksdb_engine_handler.read_list_by_model(cf, mode)?;
        let results = DashMap::with_capacity(raw.len().min(32));

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
        Ok(results)
    })
}
