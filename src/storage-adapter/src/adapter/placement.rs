use crate::storage::StorageAdapter;
use common_base::{errors::RobustMQError, log::info};

#[derive(Clone)]
pub struct PlacementStorageAdapter {}

impl PlacementStorageAdapter {
    pub fn new() -> Self {
        return PlacementStorageAdapter {};
    }
}

impl StorageAdapter for PlacementStorageAdapter {
    fn kv_set(&self, key: String, value: String) -> Result<(), RobustMQError> {
        info(format!("kv_set:{},{}", key, value));
        return Ok(());
    }
    fn kv_get(&self, key: String) -> Result<String, RobustMQError> {
        info(format!("kv_get:{}", key));
        return Ok("".to_string());
    }
    fn kv_delete(&self, key: String) -> Result<(), RobustMQError> {
        info(format!("kv_delete:{}", key));
        return Ok(());
    }
    fn kv_exists(&self, key: String) -> Result<bool, RobustMQError> {
        info(format!("kv_exists:{}", key));
        return Ok(true);
    }

    fn stream_write(&self, shard_name: String, bytess: Vec<u8>) -> Result<u128, RobustMQError> {
        info(format!("stream_write:"));
        return Ok(1);
    }

    fn stream_read_next(&self, shard_name: String) -> Result<Vec<u8>, RobustMQError> {
        info(format!("stream_read"));
        return Ok("".to_string().into_bytes());
    }
    fn stream_read_next_batch(
        &self,
        shard_name: String,
        record_num: u16,
    ) -> Result<Vec<Vec<u8>>, RobustMQError> {
        info(format!("stream_read"));
        return Ok(Vec::new());
    }

    fn stream_read_by_id(
        &self,
        shard_name: String,
        record_id: u128,
    ) -> Result<Vec<u8>, RobustMQError> {
        return Ok("".to_string().into_bytes());
    }

    fn stream_read_by_timestamp(
        &self,
        shard_name: String,
        start_timestamp: u128,
        end_timestamp: u128,
    ) -> Result<Vec<Vec<u8>>, RobustMQError> {
        return Ok(Vec::new());
    }

    fn stream_read_by_last_expiry(
        &self,
        shard_name: String,
        second: u128,
    ) -> Result<Vec<Vec<u8>>, RobustMQError> {
        return Ok(Vec::new());
    }
}
