use metadata_struct::storage::adapter_record::AdapterWriteRecord;

use crate::memory::engine::MemoryStorageEngine;

impl MemoryStorageEngine {
    pub fn save_index(&self, shard_name: &str, offset: u64, msg: &AdapterWriteRecord) {
        // key index
        if let Some(key) = &msg.key {
            let key_map = self.key_index.entry(shard_name.to_string()).or_default();
            key_map.insert(key.clone(), offset);
        }

        // tag index
        if let Some(tags) = &msg.tags {
            let tag_map = self.tag_index.entry(shard_name.to_string()).or_default();
            for tag in tags.iter() {
                tag_map.entry(tag.clone()).or_default().push(offset);
            }
        }

        // timestamp index
        if msg.timestamp > 0 && offset.is_multiple_of(5000) {
            let timestamp_map = self
                .timestamp_index
                .entry(shard_name.to_string())
                .or_default();
            if !timestamp_map.contains_key(&msg.timestamp) {
                timestamp_map.insert(msg.timestamp, offset);
            }
        }
    }

    pub fn remove_indexes(&self, shard_key: &str) {
        self.tag_index.remove(shard_key);
        self.key_index.remove(shard_key);
        self.timestamp_index.remove(shard_key);
    }
}
