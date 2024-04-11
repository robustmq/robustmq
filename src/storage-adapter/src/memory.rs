use crate::storage::StorageAdapter;
use axum::async_trait;
use common_base::{errors::RobustMQError, log::info};

#[derive(Clone)]
pub struct MemoryStorageAdapter {}

impl MemoryStorageAdapter {
    pub fn new() -> Self {
        return MemoryStorageAdapter {};
    }
}

#[async_trait]
impl StorageAdapter for MemoryStorageAdapter {
    async fn kv_set(&self, key: String, value: String) -> Result<(), RobustMQError> {
        return Ok(());
    }
    async fn kv_get(&self, key: String) -> Result<String, RobustMQError> {
        info(format!("kv_get:{}", key));
        return Ok("".to_string());
    }
    async fn kv_delete(&self, key: String) -> Result<(), RobustMQError> {
        info(format!("kv_delete:{}", key));
        return Ok(());
    }
    async fn kv_exists(&self, key: String) -> Result<bool, RobustMQError> {
        info(format!("kv_exists:{}", key));
        return Ok(true);
    }

    async fn stream_write(
        &self,
        shard_name: String,
        bytess: Vec<u8>,
    ) -> Result<u128, RobustMQError> {
        info(format!("stream_write:"));
        return Ok(1);
    }

    async fn stream_read_next(&self, shard_name: String) -> Result<Vec<u8>, RobustMQError> {
        info(format!("stream_read"));
        return Ok("".to_string().into_bytes());
    }
    
    async fn stream_read_next_batch(
        &self,
        shard_name: String,
        record_num: u16,
    ) -> Result<Vec<Vec<u8>>, RobustMQError> {
        info(format!("stream_read"));
        return Ok(Vec::new());
    }

    async fn stream_read_by_id(
        &self,
        shard_name: String,
        record_id: u128,
    ) -> Result<Vec<u8>, RobustMQError> {
        return Ok("".to_string().into_bytes());
    }

    async fn stream_read_by_timestamp(
        &self,
        shard_name: String,
        start_timestamp: u128,
        end_timestamp: u128,
    ) -> Result<Vec<Vec<u8>>, RobustMQError> {
        return Ok(Vec::new());
    }

    async fn stream_read_by_last_expiry(
        &self,
        shard_name: String,
        second: u128,
    ) -> Result<Vec<Vec<u8>>, RobustMQError> {
        return Ok(Vec::new());
    }
}
