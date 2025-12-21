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
    core::{consts::DB_COLUMN_FAMILY_INDEX, error::StorageEngineError},
    segment::keys::offset_segment_offset,
};
use rocksdb_engine::{
    rocksdb::RocksDBEngine,
    storage::engine::{engine_get_by_engine, engine_save_by_engine},
};
use std::sync::Arc;

pub fn save_shard_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
    start_offset: u64,
) -> Result<(), StorageEngineError> {
    let key = offset_segment_offset(shard);
    Ok(engine_save_by_engine(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_INDEX,
        &key,
        start_offset,
    )?)
}

pub fn get_shard_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
) -> Result<u64, StorageEngineError> {
    let key = offset_segment_offset(shard);
    if let Some(res) =
        engine_get_by_engine::<u64>(rocksdb_engine_handler, DB_COLUMN_FAMILY_INDEX, &key)?
    {
        return Ok(res.data);
    }

    Err(StorageEngineError::NoOffsetInformation(shard.to_string()))
}
