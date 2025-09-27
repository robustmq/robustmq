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

use broker_core::rocksdb::DB_COLUMN_FAMILY_META_RAFT_LOG;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use rocksdb_engine::storage::{
    engine_delete, engine_delete_range, engine_exists, engine_get, engine_list_by_model,
    engine_prefix_list, engine_save,
};
use rocksdb_engine::warp::StorageDataWrap;
use rocksdb_engine::RocksDBEngine;
use serde::Serialize;
use std::sync::Arc;

pub fn engine_save_by_raft_log<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
    value: T,
) -> Result<(), CommonError>
where
    T: Serialize,
{
    engine_save(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META_RAFT_LOG,
        "raft_log",
        key_name,
        value,
    )
}

pub fn engine_get_by_raft_log(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<Option<StorageDataWrap>, CommonError> {
    engine_get(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META_RAFT_LOG,
        "raft_log",
        key_name,
    )
}

pub fn engine_exists_by_raft_log(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<bool, CommonError> {
    engine_exists(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META_RAFT_LOG,
        "raft_log",
        key_name,
    )
}

pub fn engine_delete_by_raft_log(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: String,
) -> Result<(), CommonError> {
    engine_delete(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META_RAFT_LOG,
        "raft_log",
        key_name,
    )
}

pub fn engine_delete_range_by_raft_log(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    from: Vec<u8>,
    to: Vec<u8>,
) -> Result<(), CommonError> {
    engine_delete_range(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META_RAFT_LOG,
        "raft_log",
        from,
        to,
    )
}

pub fn engine_list_prefix_by_raft_log(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    prefix_key_name: String,
) -> Result<Vec<StorageDataWrap>, CommonError> {
    engine_prefix_list(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META_RAFT_LOG,
        "raft_log",
        prefix_key_name,
    )
}

pub fn engine_list_mode_by_raft_log(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    mode: &rocksdb::IteratorMode,
) -> Result<DashMap<String, StorageDataWrap>, CommonError> {
    engine_list_by_model(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META_RAFT_LOG,
        "raft_log",
        mode,
    )
}
