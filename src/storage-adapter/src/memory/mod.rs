use crate::{record::Record, storage::StorageAdapter};
use axum::async_trait;
use common_base::errors::RobustMQError;
use dashmap::DashMap;

#[derive(Clone)]
pub struct MemoryStorageAdapter {
    pub memory_data: DashMap<String, Record>,
    pub shard_data: DashMap<String, Vec<Record>>,
    pub group_data: DashMap<String, usize>,
    pub key_index: DashMap<String, DashMap<String, u128>>,
}

impl MemoryStorageAdapter {
    pub fn new() -> Self {
        return MemoryStorageAdapter {
            memory_data: DashMap::with_capacity(256),
            shard_data: DashMap::with_capacity(256),
            group_data: DashMap::with_capacity(256),
            key_index: DashMap::with_capacity(256),
        };
    }
}

impl MemoryStorageAdapter {
    pub fn create_key_index(&self, shard_name: String, key: String, offset: usize) {}

    pub fn delete_value_index(&self, shard_name: String, key: String) {}

    pub fn create_timestamp_index(&self, shard_name: String, key: String, create_time: u128) {}

    pub fn delete_timestamp_index(&self, shard_name: String, key: String) {}
}

#[async_trait]
impl StorageAdapter for MemoryStorageAdapter {
    async fn kv_set(&self, key: String, value: Record) -> Result<(), RobustMQError> {
        self.memory_data.insert(key, value);
        return Ok(());
    }
    async fn kv_get(&self, key: String) -> Result<Option<Record>, RobustMQError> {
        if let Some(data) = self.memory_data.get(&key) {
            return Ok(Some(data.clone()));
        }
        return Ok(None);
    }
    async fn kv_delete(&self, key: String) -> Result<(), RobustMQError> {
        self.memory_data.remove(&key);
        return Ok(());
    }
    async fn kv_exists(&self, key: String) -> Result<bool, RobustMQError> {
        return Ok(self.memory_data.contains_key(&key));
    }

    async fn stream_write(
        &self,
        shard_name: String,
        message: Record,
    ) -> Result<usize, RobustMQError> {
        let mut shard = if let Some((_, da)) = self.shard_data.remove(&shard_name) {
            da
        } else {
            Vec::new()
        };
        shard.push(message);
        let offset = shard.len() - 1;
        self.shard_data.insert(shard_name, shard);

        // todo build timestamp index

        // todo build key index
        return Ok(offset);
    }

    async fn stream_read_next(
        &self,
        shard_name: String,
        group_id: String,
    ) -> Result<Option<Record>, RobustMQError> {
        let offset = if let Some(da) = self.group_data.get(&group_id) {
            *da
        } else {
            0
        };

        if let Some(da) = self.shard_data.get(&shard_name) {
            if let Some(value) = da.get(offset) {
                self.group_data.insert(shard_name, offset + 1);
                return Ok(Some(value.clone()));
            }
        }
        return Ok(None);
    }

    async fn stream_read_next_batch(
        &self,
        shard_name: String,
        group_id: String,
        record_num: usize,
    ) -> Result<Option<Vec<Record>>, RobustMQError> {
        let offset = if let Some(da) = self.group_data.get(&group_id) {
            *da
        } else {
            0
        };
        if let Some(da) = self.shard_data.get(&shard_name) {
            let mut cur_offset = 0;
            let mut result = Vec::new();
            for i in offset..(offset + record_num) {
                if let Some(value) = da.get(i) {
                    result.push(value.clone());
                    cur_offset += 1;
                } else {
                    break;
                }
            }
            if cur_offset > 0 {
                self.group_data.insert(shard_name, offset + cur_offset);
            }
            return Ok(Some(result));
        }
        return Ok(None);
    }

    async fn stream_read_by_offset(
        &self,
        shard_name: String,
        offset: usize,
    ) -> Result<Option<Record>, RobustMQError> {
        if let Some(da) = self.shard_data.get(&shard_name) {
            if let Some(value) = da.get(offset) {
                return Ok(Some(value.clone()));
            }
        }
        return Ok(None);
    }

    async fn stream_read_by_timestamp(
        &self,
        shard_name: String,
        start_timestamp: u128,
        end_timestamp: u128,
    ) -> Result<Option<Vec<Record>>, RobustMQError> {
        return Ok(None);
    }

    async fn stream_read_by_key(
        &self,
        shard_name: String,
        key: String,
    ) -> Result<Option<Record>, RobustMQError> {
        return Ok(None);
    }
}
