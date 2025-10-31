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
        base::{engine_delete, engine_exists, engine_get, engine_prefix_list, engine_save},
        family::DB_COLUMN_FAMILY_META,
    },
    warp::StorageDataWrap,
};
use common_base::error::common::CommonError;
use serde::Serialize;
use std::sync::Arc;

pub fn engine_save_by_meta<T>(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: &str,
    value: T,
) -> Result<(), CommonError>
where
    T: Serialize,
{
    engine_save(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META,
        "meta",
        key_name,
        value,
    )
}

pub fn engine_get_by_meta(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: &str,
) -> Result<Option<StorageDataWrap>, CommonError> {
    engine_get(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META,
        "meta",
        key_name,
    )
}

pub fn engine_exists_by_meta(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: &str,
) -> Result<bool, CommonError> {
    engine_exists(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META,
        "meta",
        key_name,
    )
}

pub fn engine_delete_by_meta(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    key_name: &str,
) -> Result<(), CommonError> {
    engine_delete(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META,
        "meta",
        key_name,
    )
}

pub fn engine_prefix_list_by_meta(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    prefix_key_name: &str,
) -> Result<Vec<StorageDataWrap>, CommonError> {
    engine_prefix_list(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_META,
        "meta",
        prefix_key_name,
    )
}
