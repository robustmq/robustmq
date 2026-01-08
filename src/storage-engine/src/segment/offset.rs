fn get_shard_offset_by_timestamp_by_segment(
    &self,
    shard_name: &str,
    timestamp: u64,
    strategy: AdapterOffsetStrategy,
) -> Result<u64, StorageEngineError> {
    if let Some(segment) =
        get_in_segment_by_timestamp(&self.cache_manager, shard_name, timestamp as i64)?
    {
        let segment_iden = SegmentIdentity::new(shard_name, segment);
        if let Some(index_data) =
            get_index_data_by_timestamp(&self.rocksdb_engine_handler, &segment_iden, timestamp)?
        {
            Ok(index_data.offset)
        } else {
            Err(StorageEngineError::CommonErrorStr(format!(
                "No index data found for timestamp {} in segment {}",
                timestamp, segment
            )))
        }
    } else {
        match strategy {
            AdapterOffsetStrategy::Earliest => get_earliest_offset(
                &self.cache_manager,
                &self.rocksdb_engine_handler,
                shard_name,
            ),
            AdapterOffsetStrategy::Latest => get_latest_offset(
                &self.rocksdb_engine_handler,
                &self.cache_manager,
                shard_name,
            ),
        }
    }
}
