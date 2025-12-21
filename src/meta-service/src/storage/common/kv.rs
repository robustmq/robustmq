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

use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_exists_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};

#[derive(Debug, Clone)]
pub struct KvStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl KvStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        KvStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn set(&self, key: String, value: String) -> Result<(), CommonError> {
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, value)
    }

    pub fn delete(&self, key: String) -> Result<(), CommonError> {
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)
    }

    pub fn get(&self, key: String) -> Result<Option<String>, CommonError> {
        if let Some(data) =
            engine_get_by_meta_metadata::<String>(&self.rocksdb_engine_handler, &key)?
        {
            return Ok(Some(data.data));
        }
        Ok(None)
    }

    pub fn exists(&self, key: String) -> Result<bool, CommonError> {
        engine_exists_by_meta_metadata(&self.rocksdb_engine_handler, &key)
    }

    pub fn get_prefix(&self, prefix: String) -> Result<Vec<String>, CommonError> {
        match engine_prefix_list_by_meta_metadata::<String>(&self.rocksdb_engine_handler, &prefix) {
            Ok(data) => {
                let mut result = Vec::new();
                for item in data {
                    result.push(item.data);
                }
                Ok(result)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocksdb_engine::storage::family::column_family_list;
    use tempfile::tempdir;

    fn setup_kv_storage() -> KvStorage {
        let temp_dir = tempdir().unwrap();
        let engine =
            RocksDBEngine::new(temp_dir.path().to_str().unwrap(), 100, column_family_list());
        KvStorage::new(Arc::new(engine))
    }

    #[test]
    fn test_set_and_get() {
        let kv = setup_kv_storage();
        kv.set("key1".to_string(), "value1".to_string()).unwrap();
        assert_eq!(
            kv.get("key1".to_string()).unwrap(),
            Some("value1".to_string())
        );
    }

    #[test]
    fn test_set_overwrite() {
        let kv = setup_kv_storage();
        kv.set("key1".to_string(), "value1".to_string()).unwrap();
        kv.set("key1".to_string(), "value2".to_string()).unwrap();
        assert_eq!(
            kv.get("key1".to_string()).unwrap(),
            Some("value2".to_string())
        );
    }

    #[test]
    fn test_get_non_existent() {
        let kv = setup_kv_storage();
        assert_eq!(kv.get("nonexistent".to_string()).unwrap(), None);
    }

    #[test]
    fn test_delete_existing() {
        let kv = setup_kv_storage();
        kv.set("key1".to_string(), "value1".to_string()).unwrap();
        kv.delete("key1".to_string()).unwrap();
        assert!(!kv.exists("key1".to_string()).unwrap());
    }

    #[test]
    fn test_delete_non_existent() {
        let kv = setup_kv_storage();
        kv.delete("nonexistent".to_string()).unwrap();
    }

    #[test]
    fn test_exists() {
        let kv = setup_kv_storage();
        assert!(!kv.exists("key1".to_string()).unwrap());
        kv.set("key1".to_string(), "value1".to_string()).unwrap();
        assert!(kv.exists("key1".to_string()).unwrap());
    }

    #[test]
    fn test_get_prefix() {
        let kv = setup_kv_storage();
        kv.set("prefix/key1".to_string(), "value1".to_string())
            .unwrap();
        kv.set("prefix/key2".to_string(), "value2".to_string())
            .unwrap();
        kv.set("other/key3".to_string(), "value3".to_string())
            .unwrap();

        let mut result = kv.get_prefix("prefix/".to_string()).unwrap();
        result.sort();
        assert_eq!(result, vec!["value1".to_string(), "value2".to_string()]);
    }

    #[test]
    fn test_get_prefix_non_existent() {
        let kv = setup_kv_storage();
        let result = kv.get_prefix("nonexistent/".to_string()).unwrap();
        assert!(result.is_empty());
    }
}
