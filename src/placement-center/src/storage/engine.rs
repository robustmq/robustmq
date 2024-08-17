// Copyright 2023 RobustMQ Team
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

use super::{
    rocksdb::{RocksDBEngine, DB_COLUMN_FAMILY_CLUSTER},
    StorageDataWrap,
};
use common_base::errors::RobustMQError;
use serde::Serialize;
use std::sync::Arc;

pub fn engine_save_by_cluster<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
    value: T,
) -> Result<(), RobustMQError>
where
    T: Serialize,
{
    return engine_save(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_CLUSTER,
        key_name,
        value,
    );
}

pub fn engine_get_by_cluster(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<Option<StorageDataWrap>, RobustMQError> {
    return engine_get(rocksdb_engine_handler, DB_COLUMN_FAMILY_CLUSTER, key_name);
}

pub fn engine_exists_by_cluster(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<bool, RobustMQError> {
    return engine_exists(rocksdb_engine_handler, DB_COLUMN_FAMILY_CLUSTER, key_name);
}

pub fn engine_delete_by_cluster(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<(), RobustMQError> {
    return engine_delete(rocksdb_engine_handler, DB_COLUMN_FAMILY_CLUSTER, key_name);
}
pub fn engine_prefix_list_by_cluster(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    prefix_key_name: String,
) -> Result<Vec<StorageDataWrap>, RobustMQError> {
    return engine_prefix_list(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_CLUSTER,
        prefix_key_name,
    );
}

fn engine_save<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rocksdb_cluster: &str,
    key_name: String,
    value: T,
) -> Result<(), RobustMQError>
where
    T: Serialize,
{
    let cf = if rocksdb_cluster.to_string() == DB_COLUMN_FAMILY_CLUSTER.to_string() {
        rocksdb_engine_handler.cf_cluster()
    } else {
        return Err(RobustMQError::ClusterNoAvailableNode);
    };

    let content = match serde_json::to_vec(&value) {
        Ok(data) => data,
        Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
    };

    let data = StorageDataWrap::new(content);
    match rocksdb_engine_handler.write(cf, &key_name, &data) {
        Ok(_) => {
            return Ok(());
        }
        Err(e) => {
            return Err(RobustMQError::CommmonError(e));
        }
    }
}

fn engine_get(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rocksdb_cluster: &str,
    key_name: String,
) -> Result<Option<StorageDataWrap>, RobustMQError> {
    let cf = if rocksdb_cluster.to_string() == DB_COLUMN_FAMILY_CLUSTER.to_string() {
        rocksdb_engine_handler.cf_cluster()
    } else {
        return Err(RobustMQError::ClusterNoAvailableNode);
    };
    match rocksdb_engine_handler.read::<StorageDataWrap>(cf, &key_name) {
        Ok(Some(data)) => {
            return Ok(Some(data));
        }
        Ok(None) => {
            return Ok(None);
        }
        Err(e) => {
            return Err(RobustMQError::CommmonError(e));
        }
    }
}

fn engine_delete(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rocksdb_cluster: &str,
    key_name: String,
) -> Result<(), RobustMQError> {
    let cf = if rocksdb_cluster.to_string() == DB_COLUMN_FAMILY_CLUSTER.to_string() {
        rocksdb_engine_handler.cf_cluster()
    } else {
        return Err(RobustMQError::ClusterNoAvailableNode);
    };

    rocksdb_engine_handler.delete(cf, &key_name)
}

fn engine_exists(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rocksdb_cluster: &str,
    key_name: String,
) -> Result<bool, RobustMQError> {
    let cf = if rocksdb_cluster.to_string() == DB_COLUMN_FAMILY_CLUSTER.to_string() {
        rocksdb_engine_handler.cf_cluster()
    } else {
        return Err(RobustMQError::ClusterNoAvailableNode);
    };

    return Ok(rocksdb_engine_handler.exist(cf, &key_name));
}

fn engine_prefix_list(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    rocksdb_cluster: &str,
    prefix_key_name: String,
) -> Result<Vec<StorageDataWrap>, RobustMQError> {
    let cf = if rocksdb_cluster.to_string() == DB_COLUMN_FAMILY_CLUSTER.to_string() {
        rocksdb_engine_handler.cf_cluster()
    } else {
        return Err(RobustMQError::ClusterNoAvailableNode);
    };

    let data_list = rocksdb_engine_handler.read_prefix(cf, &prefix_key_name);
    let mut results = Vec::new();
    for raw in data_list {
        for (_, v) in raw {
            match serde_json::from_slice::<StorageDataWrap>(v.as_ref()) {
                Ok(v) => results.push(v),
                Err(_) => {
                    continue;
                }
            }
        }
    }
    return Ok(results);
}
