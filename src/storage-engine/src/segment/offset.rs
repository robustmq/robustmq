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

use crate::{core::error::StorageEngineError, segment::keys::offset_segment_offset};
use rocksdb_engine::{
    rocksdb::RocksDBEngine,
    storage::{
        engine::{engine_get_by_engine, engine_save_by_engine},
        family::DB_COLUMN_FAMILY_STORAGE_ENGINE,
    },
};
use std::sync::Arc;

pub fn save_shard_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
    offset: u64,
) -> Result<(), StorageEngineError> {
    let key = offset_segment_offset(shard);
    Ok(engine_save_by_engine(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_STORAGE_ENGINE,
        &key,
        offset,
    )?)
}

pub fn get_shard_offset(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
) -> Result<u64, StorageEngineError> {
    let key = offset_segment_offset(shard);
    if let Some(res) = engine_get_by_engine::<u64>(
        rocksdb_engine_handler,
        DB_COLUMN_FAMILY_STORAGE_ENGINE,
        &key,
    )? {
        return Ok(res.data);
    }

    Err(StorageEngineError::NoOffsetInformation(shard.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocksdb_engine::test::test_rocksdb_instance;

    #[tokio::test]
    async fn offset_save_and_get_test() {
        let rocksdb = test_rocksdb_instance();

        let shard = "test_shard";
        let expected_offset = 12345u64;

        save_shard_offset(&rocksdb, shard, expected_offset).unwrap();

        let actual_offset = get_shard_offset(&rocksdb, shard).unwrap();

        assert_eq!(actual_offset, expected_offset);

        save_shard_offset(&rocksdb, shard, 99999).unwrap();
        let updated_offset = get_shard_offset(&rocksdb, shard).unwrap();
        assert_eq!(updated_offset, 99999);

        let result = get_shard_offset(&rocksdb, "non_existent_shard");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StorageEngineError::NoOffsetInformation(_)
        ));
    }
}
