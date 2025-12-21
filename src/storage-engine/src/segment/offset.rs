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
