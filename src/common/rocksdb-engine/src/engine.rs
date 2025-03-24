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
use dashmap::DashMap;
use serde::Serialize;

use crate::warp::StorageDataWrap;
use crate::RocksDBEngine;

pub fn rocksdb_engine_save<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    comlumn_family: &str,
    key_name: String,
    value: T,
) -> Result<(), CommonError>
where
    T: Serialize,
{
    let cf = if let Some(cf) = rocksdb_engine_handler.cf_handle(comlumn_family) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            comlumn_family.to_string(),
        ));
    };

    let content = match serde_json::to_string(&value) {
        Ok(data) => data,
        Err(e) => return Err(CommonError::CommonError(e.to_string())),
    };

    let data = StorageDataWrap::new(content);
    match rocksdb_engine_handler.write(cf, &key_name, &data) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn rocksdb_engine_get(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    comlumn_family: &str,
    key_name: String,
) -> Result<Option<StorageDataWrap>, CommonError> {
    let cf = if let Some(cf) = rocksdb_engine_handler.cf_handle(comlumn_family) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            comlumn_family.to_string(),
        ));
    };

    rocksdb_engine_handler.read::<StorageDataWrap>(cf, &key_name)
}

pub fn rocksdb_engine_delete(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    comlumn_family: &str,
    key_name: String,
) -> Result<(), CommonError> {
    let cf = if let Some(cf) = rocksdb_engine_handler.cf_handle(comlumn_family) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            comlumn_family.to_string(),
        ));
    };

    rocksdb_engine_handler.delete(cf, &key_name)
}

pub fn rocksdb_engine_exists(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    comlumn_family: &str,
    key_name: String,
) -> Result<bool, CommonError> {
    let cf = if let Some(cf) = rocksdb_engine_handler.cf_handle(comlumn_family) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            comlumn_family.to_string(),
        ));
    };

    Ok(rocksdb_engine_handler.exist(cf, &key_name))
}

pub fn rocksdb_engine_prefix_list(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    comlumn_family: &str,
    prefix_key_name: String,
) -> Result<Vec<StorageDataWrap>, CommonError> {
    let cf = if let Some(cf) = rocksdb_engine_handler.cf_handle(comlumn_family) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            comlumn_family.to_string(),
        ));
    };

    let data_list = rocksdb_engine_handler.read_prefix(cf, &prefix_key_name);
    let mut results = Vec::new();
    if let Ok(raw) = data_list {
        for (_, v) in raw {
            match serde_json::from_slice::<StorageDataWrap>(v.as_ref()) {
                Ok(v) => results.push(v),
                Err(_) => {
                    continue;
                }
            }
        }
    }
    Ok(results)
}

pub fn rocksdb_engine_prefix_map(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    comlumn_family: &str,
    prefix_key_name: String,
) -> Result<DashMap<String, StorageDataWrap>, CommonError> {
    let cf = if let Some(cf) = rocksdb_engine_handler.cf_handle(comlumn_family) {
        cf
    } else {
        return Err(CommonError::RocksDBFamilyNotAvailable(
            comlumn_family.to_string(),
        ));
    };

    let data_list = rocksdb_engine_handler.read_prefix(cf, &prefix_key_name);
    let results = DashMap::with_capacity(2);
    if let Ok(raw) = data_list {
        for (key, v) in raw {
            match serde_json::from_slice::<StorageDataWrap>(v.as_ref()) {
                Ok(v) => {
                    results.insert(key, v);
                }
                Err(_) => {
                    continue;
                }
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
        engine::{
            rocksdb_engine_delete, rocksdb_engine_exists, rocksdb_engine_get, rocksdb_engine_save,
        },
        RocksDBEngine,
    };

    #[tokio::test]
    async fn base_rw_str_test() {
        let path = "/tmp/test";
        let rocksdb_engine_handler =
            Arc::new(RocksDBEngine::new(path, 100, vec!["default".to_string()]));
        let key = "test_key".to_string();
        let value = "test_value".to_string();
        let result = rocksdb_engine_save(
            rocksdb_engine_handler.clone(),
            "default",
            key.clone(),
            value,
        );
        assert!(result.is_ok());
        let result = rocksdb_engine_get(rocksdb_engine_handler.clone(), "default", key.clone());
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        let result = rocksdb_engine_delete(rocksdb_engine_handler.clone(), "default", key.clone());
        assert!(result.is_ok());
        let result = rocksdb_engine_exists(rocksdb_engine_handler.clone(), "default", key.clone());
        assert!(result.is_ok());
        assert!(!result.unwrap());
        remove_dir_all(path).unwrap();
    }

    #[tokio::test]
    async fn base_rw_session_test() {
        let path = "/tmp/test";
        let rocksdb_engine_handler =
            Arc::new(RocksDBEngine::new(path, 100, vec!["default".to_string()]));
        let key = "test_key".to_string();
        let value = MqttSession::default();
        let result = rocksdb_engine_save(
            rocksdb_engine_handler.clone(),
            "default",
            key.clone(),
            value,
        );
        assert!(result.is_ok());
        let result = rocksdb_engine_get(rocksdb_engine_handler.clone(), "default", key.clone());
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        let result = rocksdb_engine_delete(rocksdb_engine_handler.clone(), "default", key.clone());
        assert!(result.is_ok());
        let result = rocksdb_engine_exists(rocksdb_engine_handler.clone(), "default", key.clone());
        assert!(result.is_ok());
        assert!(!result.unwrap());
        remove_dir_all(path).unwrap();
    }
}
