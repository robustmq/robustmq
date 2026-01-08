use crate::{
    commitlog::{
        memory::engine::MemoryStorageEngine,
        offset::{get_earliest_offset, save_earliest_offset},
    },
    core::error::StorageEngineError,
};
use dashmap::DashMap;
use metadata_struct::storage::storage_record::StorageRecord;

impl MemoryStorageEngine {
    pub fn try_remove_old_data(
        &self,
        shard_name: &str,
        current_shard_data: &DashMap<u64, StorageRecord>,
        new_message_count: usize,
    ) -> Result<(), StorageEngineError> {
        let next_num = current_shard_data.len() + new_message_count;
        if next_num > self.config.max_records_per_shard {
            let offset = get_earliest_offset(
                &self.cache_manager,
                &self.rocksdb_engine_handler,
                shard_name,
            )?;
            let discard_num = (current_shard_data.len() as f64 * 0.2) as u64;

            if let Some(key_map) = self.key_index.get_mut(shard_name) {
                for i in offset..(offset + discard_num) {
                    if let Some((_, record)) = current_shard_data.remove(&i) {
                        if let Some(key) = &record.metadata.key {
                            key_map.remove(key);
                        }
                    }
                }
            } else {
                for i in offset..(offset + discard_num) {
                    current_shard_data.remove(&i);
                }
            }

            let new_earliest = offset + discard_num;
            save_earliest_offset(
                &self.cache_manager,
                &self.rocksdb_engine_handler,
                shard_name,
                new_earliest,
            )?;
            self.cleanup_indexes_by_offset(shard_name, new_earliest);
        }
        Ok(())
    }

    fn cleanup_indexes_by_offset(&self, shard_name: &str, earliest_offset: u64) {
        if let Some(tag_map) = self.tag_index.get_mut(shard_name) {
            for mut offsets in tag_map.iter_mut() {
                offsets.value_mut().retain(|&o| o >= earliest_offset);
            }
            tag_map.retain(|_, v| !v.is_empty());
        }

        if let Some(key_map) = self.key_index.get_mut(shard_name) {
            key_map.retain(|_, &mut o| o >= earliest_offset);
        }

        if let Some(ts_map) = self.timestamp_index.get_mut(shard_name) {
            ts_map.retain(|_, &mut o| o >= earliest_offset);
        }
    }
}
