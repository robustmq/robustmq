use common_base::{errors::RobustMQError, log::info};

use crate::storage::StorageAdapter;

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

    fn stream_write(&self) -> Result<(), RobustMQError> {
        info(format!("stream_write:"));
        return Ok(());
    }
    fn stream_read(&self) -> Result<String, RobustMQError> {
        info(format!("stream_read"));
        return Ok("".to_string());
    }
}
