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
use common_base::error::common::CommonError;
use dashmap::DashMap;
use rocksdb::BoundColumnFamily;
use serde::Serialize;
use std::sync::Arc;

/// Helper function to get column family handle
#[inline]
fn get_cf_handle<'a>(
    engine: &'a RocksDBEngine,
    column_family: &str,
) -> Result<Arc<BoundColumnFamily<'a>>, CommonError> {
    engine
        .cf_handle(column_family)
        .ok_or_else(|| CommonError::RocksDBFamilyNotAvailable(column_family.to_string()))
}

pub fn rocksdb_engine_save<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    key_name: &str,
    value: T,
) -> Result<(), CommonError>
where
    T: Serialize,
{
    let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;

    let content =
        serde_json::to_string(&value).map_err(|e| CommonError::CommonError(e.to_string()))?;

    let data = StorageDataWrap::new(content);
    rocksdb_engine_handler.write(cf, key_name, &data)?;
    Ok(())
}

pub fn rocksdb_engine_get(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    key_name: &str,
) -> Result<Option<StorageDataWrap>, CommonError> {
    let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
    rocksdb_engine_handler.read::<StorageDataWrap>(cf, key_name)
}

pub fn rocksdb_engine_delete(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    key_name: &str,
) -> Result<(), CommonError> {
    let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
    rocksdb_engine_handler.delete(cf, key_name)
}

pub fn rocksdb_engine_delete_range(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    from: Vec<u8>,
    to: Vec<u8>,
) -> Result<(), CommonError> {
    let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
    rocksdb_engine_handler.delete_range_cf(cf, from, to)
}

pub fn rocksdb_engine_delete_prefix(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    prefix_key: &str,
) -> Result<(), CommonError> {
    let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
    rocksdb_engine_handler.delete_prefix(cf, prefix_key)
}

pub fn rocksdb_engine_exists(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    key_name: &str,
) -> Result<bool, CommonError> {
    let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;
    Ok(rocksdb_engine_handler.exist(cf, key_name))
}

pub fn rocksdb_engine_list_by_prefix(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    prefix_key_name: &str,
) -> Result<Vec<StorageDataWrap>, CommonError> {
    let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;

    let raw = rocksdb_engine_handler.read_prefix(cf, prefix_key_name)?;
    let mut results = Vec::with_capacity(raw.len().min(64));

    for (_key, v) in raw {
        match serde_json::from_slice::<StorageDataWrap>(v.as_ref()) {
            Ok(v) => results.push(v),
            Err(_e) => {
                // Silently skip deserialization errors
                continue;
            }
        }
    }
    Ok(results)
}

pub fn rocksdb_engine_list_by_mode(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    mode: &rocksdb::IteratorMode,
) -> Result<DashMap<String, StorageDataWrap>, CommonError> {
    let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;

    let raw = rocksdb_engine_handler.read_list_by_model(cf, mode)?;
    let results = DashMap::with_capacity(raw.len().min(32));

    for (key, v) in raw {
        match serde_json::from_slice::<StorageDataWrap>(v.as_ref()) {
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
}

pub fn rocksdb_engine_list_by_prefix_to_map(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    column_family: &str,
    prefix_key_name: &str,
) -> Result<DashMap<String, StorageDataWrap>, CommonError> {
    let cf = get_cf_handle(&rocksdb_engine_handler, column_family)?;

    let raw = rocksdb_engine_handler.read_prefix(cf, prefix_key_name)?;
    let results = DashMap::with_capacity(raw.len().min(32));

    for (key, v) in raw {
        match serde_json::from_slice::<StorageDataWrap>(v.as_ref()) {
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
}

#[cfg(test)]
mod tests {
    use std::{fs::remove_dir_all, sync::Arc};

    use metadata_struct::mqtt::session::MqttSession;

    use crate::{
        rocksdb::RocksDBEngine,
        storage::engine::{
            rocksdb_engine_delete, rocksdb_engine_exists, rocksdb_engine_get, rocksdb_engine_save,
        },
    };

    use tempfile::tempdir;

    #[tokio::test]
    async fn base_rw_str_test() {
        let path = tempdir().unwrap().path().to_str().unwrap().to_string();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            path.as_str(),
            100,
            vec!["default".to_string()],
        ));
        let key = "test_key";
        let value = "test_value".to_string();
        let result = rocksdb_engine_save(rocksdb_engine_handler.clone(), "default", key, value);
        assert!(result.is_ok());
        let result = rocksdb_engine_get(rocksdb_engine_handler.clone(), "default", key);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        let result = rocksdb_engine_delete(rocksdb_engine_handler.clone(), "default", key);
        assert!(result.is_ok());
        let result = rocksdb_engine_exists(rocksdb_engine_handler.clone(), "default", key);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        remove_dir_all(path).unwrap();
    }

    #[tokio::test]
    async fn base_rw_session_test() {
        let path = tempdir().unwrap().path().to_str().unwrap().to_string();
        let rocksdb_engine_handler = Arc::new(RocksDBEngine::new(
            path.as_str(),
            100,
            vec!["default".to_string()],
        ));
        let key = "test_key";
        let value = MqttSession::default();
        let result = rocksdb_engine_save(rocksdb_engine_handler.clone(), "default", key, value);
        assert!(result.is_ok());
        let result = rocksdb_engine_get(rocksdb_engine_handler.clone(), "default", key);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        let result = rocksdb_engine_delete(rocksdb_engine_handler.clone(), "default", key);
        assert!(result.is_ok());
        let result = rocksdb_engine_exists(rocksdb_engine_handler.clone(), "default", key);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        remove_dir_all(path).unwrap();
    }
}
